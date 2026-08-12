pub mod cmd;
pub mod frame;
pub mod mapping;

use crate::comms::pdo::cmd::PdoCommand;
use crate::comms::pdo::mapping::PDOSet;
use crate::comms::pdo::mapping::PdoMapping;
use crate::comms::pdo::mapping::PdoType;
use crate::comms::pdo::mapping::custom::RPDO_CONTROL_OPMODE;
use crate::comms::pdo::mapping::custom::RPDO_IDX_CONTROL_WORD;
use crate::comms::pdo::mapping::custom::RPDO_IDX_TARGET_POS;
use crate::comms::pdo::mapping::custom::RPDO_IDX_TARGET_TORQUE;
use crate::comms::pdo::mapping::custom::RPDO_IDX_TARGET_VEL;
use crate::comms::pdo::mapping::custom::RPDO_TARGET_POS;
use crate::comms::pdo::mapping::custom::RPDO_TARGET_TORQUE;
use crate::comms::pdo::mapping::custom::RPDO_TARGET_VEL;
use crate::comms::pdo::mapping::custom::get_dlc;
use crate::comms::pdo::mapping::minimal::RPDO_CONTROL_TARGET_POS_TORQUE;
use crate::driver::cyclic::CyclicSynchronousMode;
use crate::driver::oms::cyclic_pos::CyclicPositionSetpoint;
use crate::driver::oms::cyclic_torque::CyclicTorqueSetpoint;
use crate::driver::oms::cyclic_vel::CyclicVelocitySetpoint;
use crate::driver::oms::home::*;
use crate::driver::oms::position::*;
use crate::driver::oms::setpoint::Setpoint;
use crate::driver::oms::torque::*;
use crate::driver::oms::velocity::*;
use crate::error::InitialisationError;
use crate::od;
use std::time::Duration;

/// PDO based Cia402Transport impl for oze-canopen
use oze_canopen::{interface::CanOpenInterface, transmitter::TxPacket};
use tokio::sync::mpsc;
use tokio::task::AbortHandle;
use tracing::*;

use crate::{
    comms::pdo::frame::PdoFrame,
    driver::{oms::OperationMode, state::Cia402Flags, update::ControlWord},
    error::DriveError,
    od::entry::ODEntry,
};

const SEND_TIMEOUT: Duration = Duration::from_millis(100);
const DEFAULT_MODE: OperationMode = OperationMode::ProfileVelocity;

/// Low level CANopen PDO transport implementation
/// Manages PDO communication to a single node_id / motor in a seperate task
/// Used by the update publisher & Setpoint Manager
pub struct Pdo {
    canopen: CanOpenInterface,
    node_id: u8,
    default_pdo_set: &'static PDOSet,
    minimal_pdo_set: &'static PDOSet,
    default_dlcs: [usize; 8],
    minimal_dlcs: [usize; 8],
    rpdo_frames: [PdoFrame; 4],
    pdo_rx: mpsc::Receiver<PdoCommand>,
    mode: OperationMode,
}

impl Pdo {
    pub fn init(
        canopen: CanOpenInterface,
        node_id: u8,
        default_pdo_set: &'static PDOSet,
        minimal_pdo_set: &'static PDOSet,
        set: &mut tokio::task::JoinSet<()>,
    ) -> Result<(AbortHandle, mpsc::Sender<PdoCommand>), InitialisationError> {
        // Check if all required mappings are present
        Pdo::check_required_rpdo_mappings(default_pdo_set.rpdos)?;

        // Initialize communication channels
        let (pdo_tx, pdo_rx) = tokio::sync::mpsc::channel(10);

        // Calculate RPDO Data Length Codes
        let mut minimal_dlcs = [0usize; 8];
        for (idx, mappings) in minimal_pdo_set.rpdos.iter().enumerate() {
            minimal_dlcs[idx] = get_dlc(mappings);
        }
        let mut default_dlcs = [0usize; 8];
        for (idx, mappings) in default_pdo_set.rpdos.iter().enumerate() {
            default_dlcs[idx] = get_dlc(mappings);
        }

        let pdo = Self {
            canopen,
            node_id,
            default_pdo_set,
            minimal_pdo_set,
            default_dlcs,
            minimal_dlcs,
            // Initialize RPDO Frames with default RPDO mapping DLCs
            rpdo_frames: core::array::from_fn(|idx| PdoFrame::with_dlc(default_dlcs[idx])),
            pdo_rx,
            mode: DEFAULT_MODE,
        };

        // Run the PDO communication task
        let handle = set.spawn(pdo.run());

        Ok((handle, pdo_tx))
    }

    // PDO communication task routine
    async fn run(mut self) {
        loop {
            // Await a new PDO command
            match self.pdo_rx.recv().await {
                Some(cmd) => {
                    // Handle received commands
                    use PdoCommand::*;
                    match cmd {
                        UpdateCia402Flags(cia402flags) => {
                            self.update_cia402_state_transition(&cia402flags);
                        }
                        WriteCia402Transition(cia402flags) => {
                            if let Err(err) = self.write_cia402_state_transition(&cia402flags).await
                            {
                                error!(
                                    "PDO unable to write cia402 state transition: 
                                    {cia402flags:?} to device id {} - {err}",
                                    self.node_id
                                );
                            }
                        }
                        WriteSetpoint(setpoint) => {
                            if let Err(err) = self.write_setpoint(&setpoint).await {
                                error!(
                                    "PDO unable to send setpoint: {setpoint:?} 
                                    to device id {} - {err}",
                                    self.node_id
                                );
                            }
                        }
                        SwitchToCyclicSynchronousMode(mode) => {
                            if let Err(err) =
                                self.enable_cyclic_synchronous_mode(mode.clone()).await
                            {
                                error!(
                                    "PDO unable to enable Cyclic Synchronous Mode: {mode:?} 
                                    for device id {} - {err}",
                                    self.node_id
                                );
                            }
                        }
                        ExitCyclicSynchronousMode => {
                            if let Err(err) = self.disable_cyclic_synchronous_mode().await {
                                error!("Unable to disable cyclic synchronous mode: {err}");
                            }
                        }
                        UpdatePositionSetpointFlags(position_flags_cw) => {
                            if let Err(err) =
                                self.update_position_setpoint(&position_flags_cw).await
                            {
                                error!("Unable to disable cyclic synchronous mode: {err}");
                            }
                        }
                        UpdateHomingSetpointFlags(home_flags_cw) => {
                            if let Err(err) = self.update_homing_setpoint(&home_flags_cw).await {
                                error!("Unable to disable cyclic synchronous mode: {err}");
                            }
                        }
                    }
                }
                _ => {
                    error!(
                        "PDO receiver channel closed -> update publisher and 
                        setpoint manager both dropped their pdo_tx,
                        nothing to do but continue.."
                    );
                }
            }
        }
    }

    /// Updates the cia402 Controlword flags to a new value
    /// This is required when the device informs us of a state update
    /// or when we want to effect a cia402 transition ourselves
    pub fn update_cia402_state_transition(&mut self, flags: &Cia402Flags) {
        trace!("Cia402 CW Flags are updated to: {flags:?}");

        // Set the cia402 controlword bits to represent the requested state
        let mut cw = self.get_current_controlword();
        trace!("Fetched current controlword: {cw:?}");

        cw = cw.with_cia402_flags(flags);
        self.set_controlword_rpdo(cw);
    }

    // Perform the given cia402 state transition by writing the corresponding controlword flags and
    // sending the PDO that has controlword mapped out to the device
    pub async fn write_cia402_state_transition(
        &mut self,
        flags: &Cia402Flags,
    ) -> Result<(), DriveError> {
        trace!("cia402 state transition requested - flags {flags:?}");

        // Update the cia402 CW flags
        self.update_cia402_state_transition(flags);

        // Send RPDO containing updated controlword over the wire
        match self.send_rpdo(RPDO_CONTROL_OPMODE).await {
            Ok(_) => {
                trace!("RPDO1 sent to effect cia402 transition");
            }
            Err(err) => {
                error!("ERR: {err}");
                return Err(err);
            }
        }

        Ok(())
    }

    /// Set opmode to requested Cyclic Synchronous mode
    pub async fn enable_cyclic_synchronous_mode(
        &mut self,
        mode: CyclicSynchronousMode,
    ) -> Result<(), DriveError> {
        let mode: OperationMode = mode.into();
        self.mode = mode;
        trace!("Enabling Cyclic Synchronous mode: {mode:?}");

        // Set Position Mode
        self.set_operational_mode(self.mode);

        // Send RPDO1
        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        // flush RPDO2 frame to avoid stale writes
        self.rpdo_frames[1] = PdoFrame::with_dlc(self.minimal_dlcs[1]);

        Ok(())
    }

    pub async fn disable_cyclic_synchronous_mode(&mut self) -> Result<(), DriveError> {
        trace!("Disabling Cyclic Synchronous mode, switching to default mode: {DEFAULT_MODE:?}");
        self.mode = DEFAULT_MODE;

        // Set Default mode
        self.set_operational_mode(self.mode);

        // Send RPDO1 to effect mode switch
        trace!("Sending RPDO1 to effect mode switch");
        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        // When changing to cyclic Synchronous mode, T/RPDO2 get remapped
        // flush RPDO2 frame to avoid stale writes
        self.rpdo_frames[1] = PdoFrame::with_dlc(self.default_dlcs[1]);

        Ok(())
    }

    pub async fn write_setpoint(&mut self, setpoint: &Setpoint) -> Result<(), DriveError> {
        use Setpoint::*;
        match (setpoint, self.mode.is_cyclic_synchronous()) {
            (ProfilePosition(position_setpoint), false) => {
                self.write_position_setpoint(position_setpoint).await
            }
            (ProfileVelocity(velocity_setpoint), false) => {
                self.write_velocity_setpoint(velocity_setpoint).await
            }
            (ProfileTorque(torque_setpoint), false) => {
                self.write_torque_setpoint(torque_setpoint).await
            }
            (Home(homing_setpoint), false) => self.write_homing_setpoint(homing_setpoint).await,
            (CyclicPosition(cyclic_position_setpoint), true) => {
                self.write_cyclic_position_setpoint(cyclic_position_setpoint)
                    .await
            }
            (CyclicVelocity(cyclic_velocity_setpoint), true) => {
                self.write_cyclic_velocity_setpoint(cyclic_velocity_setpoint)
                    .await
            }
            (CyclicTorque(cyclic_torque_setpoint), true) => {
                self.write_cyclic_torque_setpoint(cyclic_torque_setpoint)
                    .await
            }
            // Fallthrough
            (setpoint, _) => Err(DriveError::PdoWrongSetpoint(setpoint.clone(), self.mode)),
        }
    }

    pub async fn write_cyclic_position_setpoint(
        &mut self,
        CyclicPositionSetpoint { abs_target }: &CyclicPositionSetpoint,
    ) -> Result<(), DriveError> {
        trace!("Writing cyclic position setpoint - target: {abs_target}");

        // We can skip writing a controlword, this is not mutated by a new setpoint in this mode
        // Write new target to appropriate RPDO frame
        let PdoType::RPDO(num) = RPDO_CONTROL_TARGET_POS_TORQUE.pdo else {
            error!(
                "Attempting cyclic pos setpoint write bhut RPDO_CONTROL_TARGET_POS_TORQUE is mapped to TPDO, unable to continue"
            );
            return Err(DriveError::PdoWrongMapping(RPDO_CONTROL_TARGET_POS_TORQUE));
        };
        // Index of target_position source in RPDO_CONTROL_TARGET_POS_TORQUE
        let pos_target_idx = 1usize;

        // 2. Construct RPDO2: set new cyclic Synchronous position target
        // TODO: hardcoded offsets
        self.rpdo_frames[num as usize].set(
            (RPDO_CONTROL_TARGET_POS_TORQUE.sources[pos_target_idx]
                .bit_range
                .start
                / 8) as usize,
            &abs_target.to_le_bytes(),
        );

        // Send the appropriate RPDO to device
        self.send_rpdo(RPDO_CONTROL_TARGET_POS_TORQUE).await?;

        Ok(())
    }

    pub async fn write_cyclic_velocity_setpoint(
        &mut self,
        CyclicVelocitySetpoint { target: _ }: &CyclicVelocitySetpoint,
    ) -> Result<(), DriveError> {
        todo!()
    }

    pub async fn write_cyclic_torque_setpoint(
        &mut self,
        CyclicTorqueSetpoint { target }: &CyclicTorqueSetpoint,
    ) -> Result<(), DriveError> {
        trace!("Writing cyclic torque setpoint - target: {target}");

        // We can skip writing a controlword, this is not mutated by a new setpoint in this mode
        // Write new target to appropriate RPDO frame
        let PdoType::RPDO(num) = RPDO_CONTROL_TARGET_POS_TORQUE.pdo else {
            error!(
                "Attempting cyclic torque setpoint write bhut RPDO_CONTROL_TARGET_POS_TORQUE is mapped to TPDO, unable to continue"
            );
            return Err(DriveError::PdoWrongMapping(RPDO_CONTROL_TARGET_POS_TORQUE));
        };
        // Index of target_position source in RPDO_CONTROL_TARGET_POS_TORQUE
        let torque_target_idx = 2usize;

        // 2. Construct RPDO2: set new cyclic Synchronous position target
        // TODO: hardcoded offsets
        self.rpdo_frames[num as usize].set(
            (RPDO_CONTROL_TARGET_POS_TORQUE.sources[torque_target_idx]
                .bit_range
                .start
                / 8) as usize,
            &target.to_le_bytes(),
        );

        // Send the appropriate RPDO to device
        self.send_rpdo(RPDO_CONTROL_TARGET_POS_TORQUE).await?;

        Ok(())
    }

    /// Writes the given position setpoint to the device by setting the appropriate operating mode,
    /// target setpoint and toggling the required controlword bits
    pub async fn write_position_setpoint(
        &mut self,
        PositionSetpoint {
            flags,
            target,
            profile_velocity,
        }: &PositionSetpoint,
    ) -> Result<(), DriveError> {
        // NOTE: Steps to write a new Profile Position setpoint:
        // 1. Write new target to 607Ah (and velocity/accel if changing)
        // 2. Write controlword with bit 4 = 0
        // 3. Write controlword with bit 4 = 1 (this edge triggers the move)
        //
        // -> It is extremely important to send out 607Ah (mapped in RPDO2) FIRST!

        // 1. Construct RPDO2: Set position and velocity target
        // TODO: hardcoded offsets
        self.rpdo_frames[RPDO_IDX_TARGET_POS].set(
            (RPDO_TARGET_POS.sources[0].bit_range.start / 8) as usize,
            &(*target as u32).to_le_bytes(),
        );
        self.rpdo_frames[RPDO_IDX_TARGET_POS].set(
            (RPDO_TARGET_POS.sources[1].bit_range.start / 8) as usize,
            &(profile_velocity.to_le_bytes()),
        );

        // Send RPDO2
        self.send_rpdo(RPDO_TARGET_POS).await?;

        // 2. Construct RPDO1: Set opmode to position and toggle control_word OMS bits
        trace!(
            "Writing position setpoint - target: {target} - profile_velocity: {profile_velocity} = flags: {flags:?}"
        );

        // Set Controlword
        let mut cw = self.get_current_controlword();

        trace!("Writing position setpoint - current Controlword: {cw:?}");
        cw = cw.with_position_flags(flags);
        trace!("Writing position setpoint - with position flags: {cw:?}");
        self.set_controlword_rpdo(cw);

        // Set Position Mode
        self.set_operational_mode(OperationMode::ProfilePosition);

        // Send RPDO1
        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        // NOTE: Step 3 Handshake is done through [`SetpointManager`] triggering
        // pdo::update_position_setpoint()

        Ok(())
    }

    // Updates the position setpoint OMS flags in the Controlword to indicate succesful handshake
    // This completes the steps required to send a new Profile Position Setpoint
    pub async fn update_position_setpoint(
        &mut self,
        flags: &PositionFlagsCW,
    ) -> Result<(), DriveError> {
        trace!(
            "Update Position setpoint after handshake with flags {:?}",
            flags
        );

        // Update Controlword with new position flags
        let mut cw = self.get_current_controlword();
        trace!("Updating position setpoint - current Controlword: {cw:?}");
        cw = cw.with_position_flags(flags);
        trace!("Updating position setpoint - with position flags: {cw:?}");
        self.set_controlword_rpdo(cw);

        // Set Position Mode
        self.set_operational_mode(OperationMode::ProfilePosition);

        // Send RPDO1 to complete handshake
        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        Ok(())
    }

    /// Writes the given velocity setpoint to the device by setting the appropriate operating mode,
    /// target setpoint and toggling the required controlword bits
    pub async fn write_velocity_setpoint(
        &mut self,
        VelocitySetpoint {
            target_velocity: target,
        }: &VelocitySetpoint,
    ) -> Result<(), DriveError> {
        // Set Velocity Mode
        self.set_operational_mode(OperationMode::ProfileVelocity);

        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        // Set position and torque target
        self.rpdo_frames[RPDO_IDX_TARGET_VEL]
            // TODO: hardcoded offset
            .set(
                (RPDO_TARGET_VEL.sources[0].bit_range.start / 8) as usize,
                &target.to_le_bytes(),
            );

        self.send_rpdo(RPDO_TARGET_VEL).await?;

        Ok(())
    }

    /// Writes the given torque setpoint to the device by setting the appropriate operating mode,
    /// target setpoint and toggling the required controlword bits
    pub async fn write_torque_setpoint(
        &mut self,
        TorqueSetpoint {
            target_torque: target,
        }: &TorqueSetpoint,
    ) -> Result<(), DriveError> {
        // Set Torque Mode
        self.set_operational_mode(OperationMode::ProfileTorque);

        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        // Set position and torque target
        self.rpdo_frames[RPDO_IDX_TARGET_TORQUE]
            // TODO: hardcoded offset
            .set(
                (RPDO_TARGET_TORQUE.sources[0].bit_range.start / 8) as usize,
                &target.to_le_bytes(),
            );

        self.send_rpdo(RPDO_TARGET_TORQUE).await?;

        Ok(())
    }

    /// Writes the given homing setpoint to the device by setting the appropriate operating mode,
    /// and toggling the required controlword bits
    pub async fn write_homing_setpoint(
        &mut self,
        HomingSetpoint { flags }: &HomingSetpoint,
    ) -> Result<(), DriveError> {
        trace!("Writing homing setpoint with flags {flags:?}");

        // 1. Construct RPDO1: Set opmode to homing and toggle control_word Homing bits
        // 1.A Set Position Mode
        self.set_operational_mode(OperationMode::Homing);

        trace!("Set Operation Mode Homing in RPDO1");

        // 1.B Set controlword homing bits
        let mut cw = self.get_current_controlword();
        trace!("Fetched current controlword: {cw:?}");

        cw = cw.with_home_flags(flags);
        self.set_controlword_rpdo(cw);

        trace!("Added homing flags to controlword: {cw:?} - sending RPDO1");

        // Send RPDO1
        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        info!("RPDO1 sent succesfully to effect homing setpoint");

        Ok(())
    }

    // Updates the homing setpoint OMS flags in the Controlword to indicate succesful handshake
    pub async fn update_homing_setpoint(&mut self, flags: &HomeFlagsCW) -> Result<(), DriveError> {
        trace!(
            "Update Homing setpoint after handshake with flags {:?}",
            flags
        );

        // Update Controlword with new homing flags
        let mut cw = self.get_current_controlword();
        trace!("Updating homing setpoint - current Controlword: {cw:?}");
        cw = cw.with_home_flags(flags);
        trace!("Updating homing setpoint - with position flags: {cw:?}");
        self.set_controlword_rpdo(cw);

        // Set Homing Mode
        self.set_operational_mode(OperationMode::Homing);

        // Send RPDO1 to complete handshake
        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        Ok(())
    }

    // Check if the RPDO mappings required for proper operation are present
    // TODO: use typestate instead -> compiler error, not runtime
    fn check_required_rpdo_mappings(
        rpdo_mapping_set: &'static [PdoMapping],
    ) -> Result<(), InitialisationError> {
        for od_entry in [&od::CONTROL_WORD, &od::SET_OPERATION_MODE] {
            if !Self::check_if_mapped(rpdo_mapping_set, od_entry) {
                return Err(InitialisationError::MissingRequiredPDOMapping(
                    od_entry.clone(),
                ));
            }
        }

        Ok(())
    }

    fn check_if_mapped(rpdo_mapping_set: &'static [PdoMapping], entry: &ODEntry) -> bool {
        for rpdo in rpdo_mapping_set {
            for source in rpdo.sources {
                if source.entry == entry {
                    return true;
                }
            }
        }

        false
    }

    /// Send a PDO frame out to the device
    async fn send_rpdo(&mut self, pdo_mapping: PdoMapping) -> Result<(), DriveError> {
        let PdoType::RPDO(num) = pdo_mapping.pdo else {
            return Err(DriveError::ViolatedInvariant(
                "Attempting to send a TPDO".to_string(),
            ));
        };

        trace!("sending RPDO #{num} - getting cob_id");

        let cob_id =
            pdo_mapping
                .pdo
                .get_pdo_cob_id(self.node_id)
                .ok_or(DriveError::ViolatedInvariant(
                    "Asked for the cob_id for PDO number: {rpdo_num} > 4".to_string(),
                ))?;

        trace!(
            "sending RPDO #{num} - cob_id: {cob_id:#0x} - updating rpdo_frames[{}]",
            num - 1
        );

        let idx = (num - 1) as usize;
        trace!(
            "sending RPDO #{num} - Constructing TxPacket from data: {:?} - dlc {}",
            self.rpdo_frames[idx].data, self.rpdo_frames[idx].dlc,
        );

        let value = TxPacket::new(
            cob_id,
            &self.rpdo_frames[idx].data[..self.rpdo_frames[idx].dlc],
        )
        .map_err(DriveError::CANOpenError)?;

        trace!("sending RPDO #{num} - TxPacket: {value:?}");

        self.canopen
            .tx
            .send_timeout(value, SEND_TIMEOUT)
            .await
            .map_err(DriveError::CanOpenTimeout)?;

        Ok(())
    }

    /// Gets current control word from internal state
    fn get_current_controlword(&self) -> ControlWord {
        let PdoType::RPDO(num) = RPDO_CONTROL_OPMODE.pdo else {
            panic!("Controlword is not mapped to RPDO");
        };
        let cw_idx = (num - 1) as usize;

        let cw_bytes = [
            self.rpdo_frames[cw_idx].data[0],
            self.rpdo_frames[cw_idx].data[1],
        ];

        ControlWord::from_bits(u16::from_le_bytes(cw_bytes)).expect(
            "unable to fetch current controlword from saved RPDO1 in write_position_setpoint",
        )
    }

    /// Saves new controlword in the appropriate RPDO frame, to be sent later
    fn set_controlword_rpdo(&mut self, cw: ControlWord) {
        let PdoType::RPDO(num) = RPDO_CONTROL_OPMODE.pdo else {
            panic!("Controlword is not mapped to RPDO");
        };
        let cw_idx = (num - 1) as usize;

        let cw_bytes = cw.bits().to_le_bytes();

        info!("setting controlword rpdo #{num} to new cw: {cw:?}");
        let cw = self.get_current_controlword();
        info!("Controlword before Set: {cw:?}");

        self.rpdo_frames[cw_idx].set(
            RPDO_CONTROL_OPMODE.sources[RPDO_IDX_CONTROL_WORD]
                .bit_range
                .start as usize,
            &cw_bytes,
        );

        let cw = self.get_current_controlword();
        info!("Controlword after Set: {cw:?}");
    }

    /// Effect a device operational mode change to the given operational mode
    fn set_operational_mode(&mut self, mode: OperationMode) {
        trace!("setting operational mode to {mode:?}");

        const OPMODE_OFFSET: usize = 2;

        let PdoType::RPDO(num) = RPDO_CONTROL_OPMODE.pdo else {
            panic!("OPMODE is not mapped to RPDO");
        };
        let idx = (num - 1) as usize;

        self.rpdo_frames[idx].set(OPMODE_OFFSET, &[mode as u8]);

        trace!(
            "Operational mode {mode:?} applied to rpdo_frame: {:?}",
            self.rpdo_frames[idx]
        );
    }
}

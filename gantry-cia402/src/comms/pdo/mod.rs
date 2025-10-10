pub mod frame;
pub mod mapping;

use crate::comms::pdo;
use crate::comms::pdo::mapping::PdoMapping;
use crate::comms::pdo::mapping::PdoType;
use crate::comms::pdo::mapping::custom::RPDO_CONTROL_OPMODE;
use crate::comms::pdo::mapping::custom::RPDO_IDX_CONTROL_WORD;
use crate::comms::pdo::mapping::custom::RPDO_IDX_OPMODE;
use crate::comms::pdo::mapping::custom::RPDO_IDX_TARGET_POS;
use crate::comms::pdo::mapping::custom::RPDO_IDX_TARGET_TORQUE;
use crate::comms::pdo::mapping::custom::RPDO_IDX_TARGET_VEL;
use crate::comms::pdo::mapping::custom::RPDO_TARGET_POS;
use crate::comms::pdo::mapping::custom::RPDO_TARGET_TORQUE;
use crate::comms::pdo::mapping::custom::RPDO_TARGET_VEL;
use crate::driver::oms::HomingSetpoint;
use crate::od;
use std::time::Duration;
use std::usize;

/// PDO based Cia402Transport impl for oze-canopen
use oze_canopen::{
    interface::{CanOpenInterface, SEND_TIMOUT},
    transmitter::TxPacket,
};

use crate::{
    comms::pdo::frame::PdoFrame,
    driver::{
        oms::{OperationMode, PositionSetpoint, TorqueSetpoint, VelocitySetpoint},
        state::Cia402Flags,
        update::ControlWord,
    },
    error::DriveError,
    od::entry::ODEntry,
};

/// Low level CANopen PDO transport implementation
/// Manages PDO communication to a single node_id / motor
/// Used by the update publisher
pub struct Pdo {
    canopen: CanOpenInterface,
    node_id: u8,
    rpdo_mapping_set: &'static [PdoMapping],
    rpdo_frames: [PdoFrame; 4],
}

impl Pdo {
    pub fn new(
        canopen: CanOpenInterface,
        node_id: u8,
        rpdo_mapping_set: &'static [PdoMapping],
    ) -> Result<Self, DriveError> {
        // Check if all required mappings are present
        Pdo::check_required_rpdo_mappings(rpdo_mapping_set)?;

        Ok(Self {
            canopen,
            node_id,
            rpdo_mapping_set,
            rpdo_frames: core::array::from_fn(|_| PdoFrame::zero()),
        })
    }

    // Perform the given cia402 state transition by writing the corresponding controlword flags and
    // sending the PDO that has controlword mapped out to the device
    pub async fn write_cia402_state_transition(
        &mut self,
        flags: Cia402Flags,
    ) -> Result<(), DriveError> {
        // Set the cia402 controlword bits to represent the requested state
        let cw = self.get_current_controlword();
        cw.with_cia402_flags(flags);
        self.set_controlword_rpdo(cw);

        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        Ok(())
    }

    // TODO: cleanup all the hardcoded addresses and offsets when you have time... I will have time
    // for that, right?
    pub async fn write_position_setpoint(
        &mut self,
        PositionSetpoint {
            flags,
            target,
            profile_velocity,
        }: PositionSetpoint,
    ) -> Result<(), DriveError> {
        // 1. Construct RPDO1: Set opmode to position and toggle control_word OMS bits

        // Set Controlword
        let cw = self.get_current_controlword();
        cw.with_position_flags(flags);
        self.set_controlword_rpdo(cw);

        // Set Position Mode
        self.set_operational_mode(OperationMode::ProfilePosition);

        // Send RPDO1
        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        // 2. Construct RPDO2: Set position and velocity target
        // TODO: hardcoded offsets
        self.rpdo_frames[RPDO_IDX_TARGET_POS].set(
            RPDO_TARGET_POS.sources[0].bit_range.start as usize,
            &(target as u32).to_be_bytes(),
        );
        self.rpdo_frames[RPDO_IDX_TARGET_POS].set(
            RPDO_TARGET_POS.sources[1].bit_range.start as usize,
            &(profile_velocity.to_be_bytes()),
        );

        // Send RPDO2
        self.send_rpdo(RPDO_TARGET_POS).await?;

        Ok(())
    }

    pub async fn write_velocity_setpoint(
        &mut self,
        VelocitySetpoint {
            target_velocity: target,
        }: VelocitySetpoint,
    ) -> Result<(), DriveError> {
        // Set Velocity Mode
        self.set_operational_mode(OperationMode::ProfileVelocity);

        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        // Set position and torque target
        self.rpdo_frames[RPDO_IDX_TARGET_VEL]
            // TODO: hardcoded offset
            .set(
                RPDO_TARGET_VEL.sources[0].bit_range.start as usize,
                &target.to_be_bytes(),
            );

        self.send_rpdo(RPDO_TARGET_VEL).await?;

        Ok(())
    }

    pub async fn write_torque_setpoint(
        &mut self,
        TorqueSetpoint {
            target_torque: target,
        }: TorqueSetpoint,
    ) -> Result<(), DriveError> {
        // Set Torque Mode
        self.set_operational_mode(OperationMode::ProfileTorque);

        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        // Set position and torque target
        self.rpdo_frames[RPDO_IDX_TARGET_TORQUE]
            // TODO: hardcoded offset
            .set(
                RPDO_TARGET_TORQUE.sources[0].bit_range.start as usize,
                &target.to_be_bytes(),
            );

        self.send_rpdo(RPDO_TARGET_TORQUE).await?;

        Ok(())
    }

    pub async fn write_homing_setpoint(
        &mut self,
        HomingSetpoint { flags }: HomingSetpoint,
    ) -> Result<(), DriveError> {
        // 1. Construct RPDO1: Set opmode to homing and toggle control_word Homing bits

        // 1.A Set Position Mode
        self.set_operational_mode(OperationMode::Homing);

        // 1.B Set controlword homing bits
        let cw = self.get_current_controlword();
        cw.with_home_flags(flags);
        self.set_controlword_rpdo(cw);

        // Send RPDO1
        self.send_rpdo(RPDO_CONTROL_OPMODE).await?;

        Ok(())
    }

    // Check if these rpdo mappings contain a controlword
    // TODO: move this into type system
    fn check_required_rpdo_mappings(
        rpdo_mapping_set: &'static [PdoMapping],
    ) -> Result<(), DriveError> {
        if Self::check_if_mapped(rpdo_mapping_set, &od::CONTROL_WORD)
            && Self::check_if_mapped(rpdo_mapping_set, &od::SET_OPERATION_MODE)
        {
            Ok(())
        } else {
            Err(DriveError::ViolatedInvariant(
                format!(
                    "RPDO mapping check failed for {rpdo_mapping_set:?} - control word is not present"
                )
                .to_string(),
            ))
        }
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

    /// Gets current control word
    fn get_current_controlword(&self) -> ControlWord {
        // TODO: hardcoded
        let cw_bytes = [
            self.rpdo_frames[RPDO_IDX_CONTROL_WORD].data[0],
            self.rpdo_frames[RPDO_IDX_CONTROL_WORD].data[1],
        ];
        ControlWord::from_bits(u16::from_be_bytes(cw_bytes)).expect(
            "unable to fetch current controlword from saved RPDO1 in write_position_setpoint",
        )
    }

    async fn send_rpdo(&mut self, pdo_mapping: PdoMapping) -> Result<(), DriveError> {
        let PdoType::RPDO(num) = pdo_mapping.pdo else {
            return Err(DriveError::ViolatedInvariant(
                "Attempting to send a TPDO".to_string(),
            ));
        };

        let cob_id = pdo_mapping
            .pdo
            .get_pdo_cob_id()
            .ok_or(DriveError::ViolatedInvariant(
                "Asked for the cob_id for PDO number: {rpdo_num} > 4".to_string(),
            ))?;

        let num = num as usize;

        self.canopen
            .tx
            .send_timeout(
                TxPacket {
                    cob_id,
                    data: self.rpdo_frames[num].data.to_vec(),
                },
                Duration::from_millis(SEND_TIMOUT),
            )
            .await
            .map_err(DriveError::CanOpenTimeout)?;

        Ok(())
    }

    /// Saves new controlword in the appropriate RPDO frame, to be sent later
    fn set_controlword_rpdo(&mut self, cw: ControlWord) {
        let PdoType::RPDO(num) = RPDO_CONTROL_OPMODE.pdo else {
            panic!("Controlword is not mapped to RPDO");
        };
        let num = num as usize;

        self.rpdo_frames[num].set(
            RPDO_CONTROL_OPMODE.sources[RPDO_IDX_CONTROL_WORD]
                .bit_range
                .start as usize,
            &cw.bits().to_be_bytes(),
        );
    }

    fn set_operational_mode(&mut self, mode: OperationMode) {
        const OPMODE_IDX: usize = 1;

        let PdoType::RPDO(num) = RPDO_CONTROL_OPMODE.pdo else {
            panic!("Controlword is not mapped to RPDO");
        };
        let num = num as usize;

        self.rpdo_frames[num].set(
            RPDO_CONTROL_OPMODE.sources[RPDO_IDX_OPMODE].bit_range.start as usize,
            &[mode as u8],
        );
    }
}

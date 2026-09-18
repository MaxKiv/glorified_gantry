pub mod error;
pub mod pdo;

use tracing::{info, trace};

use crate::{
    canopen::{
        CanOpen, CanOpenError,
        nmt::{
            NmtCommandSpecifier, NmtMonitorMessage,
            NmtState::{self, PreOperational},
        },
        od::{
            RPDO_COMMUNICATION_PARAMETER_BASE_INDEX, RPDO_COMMUNICATION_PARAMETER_DEACTIVATE_PDO,
            TPDO_COMMUNICATION_PARAMETER_BASE_INDEX, get_pdo_deactivation_od_entry,
        },
        pdo::{
            PdoType,
            mapping::{OMSNodePdoConfig, PdoMapping},
        },
        sdo::manager::{SdoCommand, SdoManager},
    },
    cia402::{Cia402Identifier, Cia402State},
    consts::pdo::NodePdoConfig,
    oms::{OperationMode, setpoint::Setpoint},
    rt::motor::error::MotorError,
};

enum MotorState {
    Idle,
    Operating,
    Reconfiguring(ReconfigState),
}

enum ReconfigState {
    Start,
    WaitingForNmtPreOp,
    WaitingForNmtOp,
    Parametrising(ParametrisingState),
}

enum SdoState {
    SendingNextSdo(SdoCommand),
    WaitingForSdoCmd(SdoCommand),
}

struct ParametrisingState {
    parameters: &'static [SdoCommand],
    currently_doing: usize,
    sdo_state: SdoState,
}

pub struct Cia402Motor {
    sdo: SdoManager,
    state: MotorState,
    pub id: Cia402Identifier,
    cia402_state: Cia402State,
    nmt: NmtState,
    canopen: CanOpen,
    opmode: OperationMode,
    setpoint: Setpoint,
    pdo_cfg: &'static NodePdoConfig,
    active_cfg: &'static OMSNodePdoConfig,
    default_parameters: &'static [SdoCommand],
}

impl Cia402Motor {
    pub fn new(
        id: Cia402Identifier,
        canopen: CanOpen,
        pdo_cfg: &'static NodePdoConfig,
        default_parameters: &'static [SdoCommand],
    ) -> Self {
        let nmt = NmtState::Bootup;
        let opmode = OperationMode::default();
        let setpoint = Setpoint::default();
        let cia402_state = Cia402State::default();
        let active_cfg = pdo_cfg.get_cfg_for_opmode(&opmode);

        Self {
            id,
            nmt,
            canopen,
            opmode,
            setpoint,
            cia402_state,
            pdo_cfg,
            active_cfg,
            sdo: todo!(),
            state: MotorState::Idle,
            default_parameters,
        }
    }

    pub fn request_nmt_command(&mut self, cmd: NmtCommandSpecifier) -> Result<(), CanOpenError> {
        self.canopen.send_nmt(cmd, &self.id)?;
        self.nmt = NmtState::from_cmd(&cmd);

        Ok(())
    }

    pub fn process_nmt_message(&mut self, msg: NmtMonitorMessage) {
        assert!(msg.node_id == self.id.node_id);
        self.nmt = msg.current_state;
    }

    /// Switches drive [`OperationMode`], remapping pdo if required
    /// NOTE: this must be called at the start of a cycle
    pub fn switch_operation_mode(&mut self, new: &OperationMode) -> Result<(), MotorError> {
        // Is a pdo remapping required?
        if switching_opmode_requires_pdo_remapping(&self.opmode, new) {
            // NOTE: this requires a bunch of sdo calls, so take care to call this at cycle start
            self.switch_pdo_config(new)?;
        }

        // NOTE: switch to new opmode, like any state changes, are done at the appropriate place
        // in the sync cycle
        self.opmode = *new;

        Ok(())
    }

    /// Sets new motor setpoint, switches operation mode if required
    pub fn new_motor_setpoint(&mut self, new_sp: Setpoint) {
        let required_opmode = new_sp.required_opmode();
        self.switch_operation_mode(&required_opmode);

        self.setpoint = new_sp.clone();
    }

    fn switch_pdo_config(&mut self, new: &OperationMode) -> Result<(), MotorError> {
        let new_pdo_cfg = self.pdo_cfg.get_cfg_for_opmode(new);

        // switch to NMT Pre-OP
        self.request_nmt_command(NmtCommandSpecifier::EnterPreOperational)
            .map_err(|e| MotorError::PdoRemapping(e))?;

        // remap PDO
        for (idx, rpdo) in new_pdo_cfg.rpdo.iter().enumerate() {
            if let Some(rpdo) = rpdo {
                self.remap_pdo(idx as u8, rpdo);
            }
        }
        for (idx, tpdo) in new_pdo_cfg.tpdo.iter().enumerate() {
            if let Some(tpdo) = tpdo {
                self.remap_pdo(idx as u8, tpdo);
            }
        }

        // switch to NMT OP
        self.request_nmt_command(NmtCommandSpecifier::StartRemoteNode)
            .map_err(|e| MotorError::PdoRemapping(e))?;

        self.active_cfg = new_pdo_cfg;
        Ok(())
    }

    pub fn default_parametrisation(&mut self) -> Result<(), MotorError> {
        // switch to NMT Pre-OP
        self.request_nmt_command(NmtCommandSpecifier::EnterPreOperational)
            .map_err(|e| MotorError::PdoRemapping(e))?;

        // Parametrise using SDO
        for sdo_cmd in self.default_parameters {
            self.canopen_tx.enqueue_sdo_cmd(sdo_cmd);
        }

        // switch to NMT OP
        self.request_nmt_command(NmtCommandSpecifier::StartRemoteNode)
            .map_err(|e| MotorError::PdoRemapping(e))?;

        Ok(())
    }

    pub fn tick(&mut self) {
        match &mut self.state {
            MotorState::Idle => {
                // Nothing to do?
                todo!()
            }

            MotorState::Operating => {
                // ?
                todo!()
            }

            MotorState::Reconfiguring(reconfig_state) => match reconfig_state {
                ReconfigState::Start => {
                    //
                    self.canopen
                        .send_nmt(NmtCommandSpecifier::EnterPreOperational, &self.id);
                    self.state = MotorState::Reconfiguring(ReconfigState::WaitingForNmtPreOp);
                }

                ReconfigState::WaitingForNmtPreOp => {
                    if self.nmt == PreOperational {
                        let param_state = ParametrisingState {
                            parameters: self.default_parameters,
                            currently_doing: 0,
                            sdo_state: SdoState::SendingNextSdo(self.default_parameters[0].clone()),
                        };
                        self.state =
                            MotorState::Reconfiguring(ReconfigState::Parametrising(param_state));
                    }
                }

                ReconfigState::Parametrising(parametrising_state) => {
                    match &parametrising_state.sdo_state {
                        SdoState::SendingNextSdo(sdo_command) => {
                            self.sdo.new_cmd(sdo_command.clone());
                            parametrising_state.sdo_state =
                                SdoState::WaitingForSdoCmd(sdo_command.clone());
                        }

                        SdoState::WaitingForSdoCmd(sdo_command) => todo!(),
                    }
                }

                ReconfigState::WaitingForNmtOp => todo!(),
            },
        }
    }

    fn remap_pdo(&self, num: u8, pdo_mapping: &PdoMapping) {
        let node = &self.id;
        let node_id = node.node_id.u8();

        // 1. Deactivate the PDO by setting the Valid Bit (bit 31) of subindex 01h of the corresponding communication parameter (e.g., 1400h:01h) to "1".
        let communication_index = match pdo_mapping.pdo {
            PdoType::RPDO => {
                calculate_pdo_index_offset(RPDO_COMMUNICATION_PARAMETER_BASE_INDEX, num)
            }
            PdoType::TPDO => {
                calculate_pdo_index_offset(TPDO_COMMUNICATION_PARAMETER_BASE_INDEX, num)
            }
        };
        info!(
            "Setting Pdo mapping {num}: {:?} for motor at node id {node_id}",
            pdo_mapping
        );

        let deactivate_pdo_od_entry = get_pdo_deactivation_od_entry(pdo_mapping.pdo, num);

        self.sdo
            .new_cmd(SdoCommand::upload(node, deactivate_pdo_od_entry));

        /// TODO: left of here
        let validate_bytes = sdo
            .lock()
            .await
            .upload(communication_index, 0x1)
            .await
            .map_err(DriveError::CanOpen)?;

        let validate_pdo = u32::from_le_bytes(
            validate_bytes
                .clone()
                .try_into()
                .map_err(DriveError::Conversion)?,
        );
        trace!(
            "0. Fetched current COB-ID: {:#0x} -> (node, RPDO base COB-ID): ({:#0x}, {:#0x})",
            validate_pdo,
            validate_pdo as u8,
            (validate_pdo & !(u8::MAX as u32)) as u16,
        );

        let invalidate_pdo = validate_pdo | (1 << 31);

        trace!(
            "1. Deactivate the PDO by setting the Valid Bit (bit 31) of subindex 01h of the
            corresponding PDO communication parameter ({}) to \"1\". -> Invalidation value: {:#0x}",
            communication_index, invalidate_pdo
        );
        let invalidate_data = invalidate_pdo.to_le_bytes();
        sdo.lock()
            .await
            .download(communication_index, 0x1, &invalidate_data)
            .await
            .map_err(DriveError::CanOpen)?;

        trace!(
            "1.B Set Transmission type to {:?}",
            pdo_mapping.transmission_type
        );
        sdo.lock()
            .await
            .download(
                communication_index,
                0x2,
                &[pdo_mapping.transmission_type.od_value()],
            )
            .await
            .map_err(DriveError::CanOpen)?;

        // Configure a periodic event to continously synchronise the driver with the latest device
        // state
        if let PdoType::TPDO(_) = pdo_mapping.pdo
            && pdo_mapping.transmission_type == TransmissionType::OnChange
        {
            // NOTE: this is a critical communication parameter to tune in order to obtain maximum CAN
            // bus bandwidth, see datasheet page 122 & 202
            const SYNCHRONISATION_PERIOD_MS: u16 = 1;
            const SYNCHRONISATION_SUB_IDX: u8 = 0x05;

            trace!(
                "1.C Transmission type is {:?} -> Configuring a periodic event to continously synchronise state in OD: {:#0x}:{} of val: {:x?}",
                pdo_mapping.transmission_type,
                communication_index,
                SYNCHRONISATION_SUB_IDX,
                SYNCHRONISATION_PERIOD_MS.to_le_bytes(),
            );

            sdo.lock()
                .await
                .download(
                    communication_index,
                    SYNCHRONISATION_SUB_IDX,
                    &SYNCHRONISATION_PERIOD_MS.to_le_bytes(),
                )
                .await
                .map_err(DriveError::CanOpen)?;
        }

        // Configure the inhibit time during which the device is unable to send TPDO updates
        // This prevents the devices from spamming the bus with constant updates (looking at u mr. torque)
        if let PdoType::TPDO(_) = pdo_mapping.pdo
            && pdo_mapping.transmission_type == TransmissionType::OnChange
        {
            // NOTE: this is a critical communication parameter to tune in order to obtain maximum CAN
            // bus bandwidth, see datasheet page 122 & 202
            const INHIBIT_TIME: u16 = 500; // = 50ms, this is in 100us blocks
            const INHIBIT_TIME_SUB_IDX: u8 = 0x03;

            trace!(
                "1.D Transmission type is {:?} -> Configuring Inhibit time for OD: {:#0x}:{} of duration: {:x?}",
                pdo_mapping.transmission_type,
                communication_index,
                INHIBIT_TIME_SUB_IDX,
                INHIBIT_TIME.to_le_bytes(),
            );

            sdo.lock()
                .await
                .download(
                    communication_index,
                    INHIBIT_TIME_SUB_IDX,
                    &INHIBIT_TIME.to_le_bytes(),
                )
                .await
                .map_err(DriveError::CanOpen)?;
        }

        // 2. Deactivate the mapping by setting subindex 00h of the corresponding mapping parameter to \"0\".,
        let mapping_index = match pdo_mapping.pdo {
            PdoType::RPDO(_) => calculate_pdo_index_offset(RPDO_MAPPING_PARAMETER_BASE_INDEX, num),
            PdoType::TPDO(_) => calculate_pdo_index_offset(TPDO_MAPPING_PARAMETER_BASE_INDEX, num),
        };
        trace!(
            "2. Deactivate the mapping by setting subindex 00h of the corresponding mapping parameter ({}) to \"0\".",
            mapping_index
        );
        let data = [0];
        sdo.lock()
            .await
            .download(mapping_index, 0x0, &data)
            .await
            .map_err(DriveError::CanOpen)?;

        trace!("3. Change the mapping in the desired subindices.");
        for (number, source) in pdo_mapping.sources.iter().enumerate() {
            let number = number + 1;

            trace!("3. Mapping #{number} to {source:?}");
            // Construct the payload: 2 bytes of OD entry to be mapped, 1 byte subindex, 1 byte with number of bits to be mapped
            let index_bytes = source.entry.index.to_be_bytes();
            let data: [u8; 4] = [
                index_bytes[0],
                index_bytes[1],
                source.entry.sub_index,
                source.bit_range.len,
            ];
            let vec: Vec<u8> = data.into_iter().rev().collect();

            sdo.lock()
                .await
                .download(mapping_index, number as u8, &vec)
                .await
                .map_err(DriveError::CanOpen)?;
        }

        trace!(
            "4. Activate the mapping by writing the number of objects that are to be mapped in subindex 00h of the corresponding mapping parameter (e.g., 1600h:00h)."
        );
        let data = [pdo_mapping.sources.len() as u8];
        sdo.lock()
            .await
            .download(mapping_index, 0x0, &data)
            .await
            .map_err(DriveError::CanOpen)?;

        trace!(
            "5. Activate the PDO by setting bit 31 of subindex 01h of the corresponding communication parameter (e.g., 1400h:01h) to \"0\"."
        );
        let validate_pdo = invalidate_pdo & !(1 << 31);
        sdo.lock()
            .await
            .download(communication_index, 0x1, &validate_pdo.to_le_bytes())
            .await
            .map_err(DriveError::CanOpen)?;

        Ok(())
    }
}

fn switching_opmode_requires_pdo_remapping(old: &OperationMode, new: &OperationMode) -> bool {
    if new.is_cyclic_synchronous() {
        *old != *new
    } else {
        old.is_cyclic_synchronous()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opmode_requires_parametrisation() {
        use OperationMode::*;

        let cases = [
            (Homing, ProfileTorque, false),
            (ProfileTorque, ProfilePosition, false),
            (ProfileTorque, CyclicSynchronousTorque, true),
            (CyclicSynchronousTorque, CyclicSynchronousTorque, false),
            (CyclicSynchronousTorque, CyclicSynchronousPosition, true),
            (CyclicSynchronousTorque, CyclicSynchronousVelocity, true),
            (CyclicSynchronousVelocity, CyclicSynchronousTorque, true),
            (CyclicSynchronousPosition, CyclicSynchronousVelocity, true),
        ];

        for (old, new, switch_expected) in cases {
            if switch_expected {
                assert_eq!(
                    switching_opmode_requires_pdo_remapping(&old, &new),
                    true,
                    "switching opmode from {:?} -> {:?} should require switch",
                    old,
                    new,
                );
            } else {
                assert_eq!(
                    switching_opmode_requires_pdo_remapping(&old, &new),
                    false,
                    "switching opmode from {:?} -> {:?} should require switch",
                    old,
                    new,
                );
            }
        }
    }
}

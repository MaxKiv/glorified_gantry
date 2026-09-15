pub mod error;
pub mod pdo;

use crate::{
    canopen::{
        CanOpen, CanOpenError,
        nmt::{NmtCommandSpecifier, NmtMonitorMessage, NmtState},
        pdo::mapping::OMSNodePdoConfig,
    },
    cia402::{Cia402Identifier, Cia402State},
    consts::pdo::NodePdoConfig,
    oms::{OperationMode, setpoint::Setpoint},
    rt::motor::error::MotorError,
};

pub struct Cia402Motor {
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
            self.remap_pdo(i, rpdo);
        }
        for (idx, tpdo) in new_pdo_cfg.tpdo.iter().enumerate() {
            self.remap_pdo(i, tpdo);
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

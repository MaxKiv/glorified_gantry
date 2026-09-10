use uom::si::angular_momentum::Units::newton_megameter_second;

use crate::{
    canopen::{
        CanOpen, CanOpenError,
        nmt::{NmtCommandSpecifier, NmtMonitorMessage, NmtState},
    },
    cia402::Cia402Identifier,
    oms::OperationMode,
};

pub struct Cia402Motor {
    id: Cia402Identifier,
    nmt: NmtState,
    canopen: CanOpen,
    opmode: OperationMode,
}

impl Cia402Motor {
    pub fn new(id: Cia402Identifier, canopen: CanOpen) -> Self {
        let nmt = NmtState::Bootup;
        let opmode = OperationMode::default();
        Self {
            id,
            nmt,
            canopen,
            opmode,
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

    pub fn switch_opmode(&mut self, new: OperationMode) {
        if switching_opmode_requires_parametrisation(&self.opmode, &new) {
            // switch to NMT Pre-OP
            self.request_nmt_command(NmtCommandSpecifier::EnterPreOperational);

            // Parametrise

            // switch to NMT OP
            self.request_nmt_command(NmtCommandSpecifier::StartRemoteNode);
        }

        // Switch to new opmode
        // TODO: somehow affect requested mode change -> PDO/SDO?
        // PDO are freshly remapped
        self.opmode = new;
    }
}

fn switching_opmode_requires_parametrisation(old: &OperationMode, new: &OperationMode) -> bool {
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
                    switching_opmode_requires_parametrisation(&old, &new),
                    true,
                    "switching opmode from {:?} -> {:?} should require switch",
                    old,
                    new,
                );
            } else {
                assert_eq!(
                    switching_opmode_requires_parametrisation(&old, &new),
                    false,
                    "switching opmode from {:?} -> {:?} should require switch",
                    old,
                    new,
                );
            }
        }
    }
}

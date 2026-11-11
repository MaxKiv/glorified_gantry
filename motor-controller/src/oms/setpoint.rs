use crate::oms::{
    OperationMode,
    cyclic_pos::CyclicPositionSetpoint,
    cyclic_torque::CyclicTorqueSetpoint,
    cyclic_vel::CyclicVelocitySetpoint,
    home::{HomeFlagsCW, HomingSetpoint},
    position::{PositionFlagsCW, PositionSetpoint},
    torque::TorqueSetpoint,
    velocity::VelocitySetpoint,
};

#[derive(Clone, Debug, Default)]
pub enum Setpoint {
    #[default]
    DeEnergised,
    ProfilePosition(PositionSetpoint),
    ProfileVelocity(VelocitySetpoint),
    ProfileTorque(TorqueSetpoint),
    Home(HomingSetpoint),
    CyclicPosition(CyclicPositionSetpoint),
    CyclicVelocity(CyclicVelocitySetpoint),
    CyclicTorque(CyclicTorqueSetpoint),
}

impl Setpoint {
    pub fn required_opmode(&self) -> OperationMode {
        match self {
            Setpoint::ProfilePosition(_) => OperationMode::ProfilePosition,
            Setpoint::ProfileVelocity(_) => OperationMode::Velocity,
            Setpoint::ProfileTorque(_) => OperationMode::ProfileTorque,
            Setpoint::Home(_) => OperationMode::Homing,
            Setpoint::CyclicPosition(_) => OperationMode::CyclicSynchronousPosition,
            Setpoint::CyclicVelocity(_) => OperationMode::CyclicSynchronousVelocity,
            Setpoint::CyclicTorque(_) => OperationMode::CyclicSynchronousTorque,
            Setpoint::DeEnergised => OperationMode::ProfileTorque,
        }
    }

    pub fn acknowledge_setpoint_received(&mut self) {
        match self {
            Setpoint::ProfilePosition(PositionSetpoint { flags, .. }) => {
                flags.remove(PositionFlagsCW::NEW_SETPOINT);
            }
            Setpoint::Home(HomingSetpoint { flags }) => {
                flags.remove(HomeFlagsCW::NEW_SETPOINT);
            }
            // Other modes don't shake hands, very rude!
            _ => {}
        }
    }

    pub fn is_cyclic_synchronous(&self) -> bool {
        use Setpoint::*;
        matches!(
            self,
            CyclicPosition(_) | CyclicVelocity(_) | CyclicTorque(_)
        )
    }
}

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

#[derive(Clone, Debug)]
pub enum Setpoint {
    ProfilePosition(PositionSetpoint),
    ProfileVelocity(VelocitySetpoint),
    ProfileTorque(TorqueSetpoint),
    Home(HomingSetpoint),
    CyclicPosition(CyclicPositionSetpoint),
    CyclicVelocity(CyclicVelocitySetpoint),
    CyclicTorque(CyclicTorqueSetpoint),
}

impl Setpoint {
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

    pub fn get_safe_setpoint_for_mode(mode: OperationMode) -> Setpoint {
        match mode {
            OperationMode::Velocity => {
                Setpoint::ProfileVelocity(VelocitySetpoint { target_velocity: 0 })
            }
            OperationMode::ProfileVelocity => {
                Setpoint::ProfileVelocity(VelocitySetpoint { target_velocity: 0 })
            }
            OperationMode::ProfileTorque => {
                Setpoint::ProfileTorque(TorqueSetpoint { target_torque: 0 })
            }
            OperationMode::Homing => Setpoint::Home(HomingSetpoint::default()),
            OperationMode::CyclicSynchronousVelocity => {
                Setpoint::CyclicVelocity(CyclicVelocitySetpoint { target: 0 })
            }
            OperationMode::CyclicSynchronousTorque => {
                Setpoint::CyclicTorque(CyclicTorqueSetpoint { target: 0 })
            }
            // NOTE: none of the other modes have a clearly defined safe setpoint
            // positional modes could use current position, but I'd prefer zero torque
            _ => Setpoint::ProfileTorque(TorqueSetpoint { target_torque: 0 }),
        }
    }
}

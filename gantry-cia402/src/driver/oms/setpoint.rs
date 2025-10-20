use crate::driver::oms::{
    cyclic_pos::CyclicPositionSetpoint, cyclic_torque::CyclicTorqueSetpoint,
    cyclic_vel::CyclicVelocitySetpoint, *,
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
            // Profile Velocity and Profile Torque modes don't shake hands, very rude!
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

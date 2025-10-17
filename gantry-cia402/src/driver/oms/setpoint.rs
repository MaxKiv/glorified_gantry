use crate::driver::oms::*;

#[derive(Clone, Debug)]
pub enum Setpoint {
    ProfilePosition(PositionSetpoint),
    ProfileVelocity(VelocitySetpoint),
    ProfileTorque(TorqueSetpoint),
    Home(HomingSetpoint),
}

impl Setpoint {
    pub fn acknowledge_setpoint_received(&mut self) {
        match self {
            Setpoint::ProfilePosition(PositionSetpoint { flags, .. }) => {
                flags.remove(PositionFlagsCW::NEW_SETPOINT);
            }
            // Setpoint::ProfileVelocity(VelocitySetpoint { flags, .. }) => {
            //     flags.remove(VelocityFlagsCW::NEW_SETPOINT);
            // }
            // Setpoint::ProfileTorque(TorqueSetpoint { flags, .. }) => {
            //     flags.remove(TorqueFlagsCW::NEW_SETPOINT);
            // }
            Setpoint::Home(HomingSetpoint { flags }) => {
                flags.remove(HomeFlagsCW::NEW_SETPOINT);
            }
            _ => {}
        }
    }
}

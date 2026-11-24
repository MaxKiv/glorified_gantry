use crate::oms::home::HomingSetpoint;
use crate::oms::setpoint::Setpoint;
use crate::oms::torque::TorqueSetpoint;

#[derive(Debug, Clone, PartialEq)]
pub struct GantrySetpoint {
    pub x: Option<Setpoint>,
    pub y: Option<Setpoint>,
    pub z: Option<Setpoint>,
}

impl GantrySetpoint {
    pub fn new_home_all_axis() -> Self {
        GantrySetpoint {
            x: Some(Setpoint::Home(HomingSetpoint::default())),
            y: Some(Setpoint::Home(HomingSetpoint::default())),
            z: Some(Setpoint::Home(HomingSetpoint::default())),
        }
    }

    pub fn estop_setpoint() -> Self {
        GantrySetpoint {
            x: Some(Setpoint::ProfileTorque(TorqueSetpoint { target_torque: 0 })),
            y: Some(Setpoint::ProfileTorque(TorqueSetpoint { target_torque: 0 })),
            z: Some(Setpoint::ProfileTorque(TorqueSetpoint { target_torque: 0 })),
        }
    }
}

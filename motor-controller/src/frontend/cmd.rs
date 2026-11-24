use crate::oms::home::HomingSetpoint;
use crate::oms::setpoint::Setpoint;
use crate::oms::torque::TorqueSetpoint;

use uom::si::f64::Length;
use uom::si::f64::Torque;
use uom::si::f64::Velocity;

#[derive(Debug, Clone, PartialEq)]
pub struct PositionCommand {
    target: Length,
    vel: Velocity,
    relative: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VelocityCommand {
    target: Velocity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TorqueCommand {
    target: Torque,
}

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

#[derive(Debug, Clone, Default, PartialEq)]
pub enum GantryCommand {
    /// Motors are Responsible for interpolation
    Setpoint(GantrySetpoint),
    /// You are Responsible for interpolation
    CyclicSetpoint(GantrySetpoint),
    /// Home motors
    Home,
    /// De Energise motors
    #[default]
    Idle,
    // ResetFaults, ?
}

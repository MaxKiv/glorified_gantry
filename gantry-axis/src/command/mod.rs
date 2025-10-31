pub mod handler;

use crate::axis::setpoint::*;

// Commands that can be sent to gantry
#[derive(Debug, Clone)]
pub enum GantryCommand {
    /// Provide a new setpoint for the x,y,z axis of the gantry
    Setpoint {
        x: Option<AxisSetpoint>,
        y: Option<AxisSetpoint>,
        z: Option<AxisSetpoint>,
    },

    /// Home all gantry axi
    Home,
}

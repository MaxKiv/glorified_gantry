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

impl GantryCommand {
    pub fn map_axes<F, T>(&self, mut f: F) -> Option<[Option<T>; 3]>
    where
        F: FnMut(&AxisSetpoint) -> T,
    {
        match self {
            GantryCommand::Home => None,
            GantryCommand::Setpoint { x, y, z } => Some([
                x.as_ref().map(&mut f),
                y.as_ref().map(&mut f),
                z.as_ref().map(&mut f),
            ]),
        }
    }
}

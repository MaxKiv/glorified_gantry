use crate::setpoint::*;

// Commands that can be sent to gantry
#[derive(Debug, Clone)]
pub enum GantryCommand {
    /// Move to an absolute position (in device units, e.g. encoder ticks)
    SetAbsolutePosition { setpoint: PositionSetpoint },

    /// Move relative to current position
    SetRelativePosition { setpoint: PositionSetpoint },

    /// Set continuous velocity
    SetVelocity { setpoint: VelocitySetpoint },

    /// Set continuous velocity
    SetTorque { setpoint: TorqueSetpoint },
}

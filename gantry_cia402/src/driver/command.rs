// Commands that can be sent to the motor
#[derive(Debug)]
pub enum MotorCommand {
    /// Move to an absolute position (in device units, e.g. encoder ticks)
    MoveAbsolute {
        target: i32,
        profile_velocity: Option<u32>,
    },

    /// Move relative to current position
    MoveRelative {
        delta: i32,
        profile_velocity: Option<u32>,
    },

    /// Set continuous velocity
    SetVelocity { target_velocity: i32 },

    /// Set continuous velocity
    SetTorque { target_torque: i32 },

    /// Halt immediately (stop but remain enabled)
    Halt,

    /// Disable drive (turn off power stage)
    Disable,

    /// Enable drive (transition to operation enabled)
    Enable,
    // /// Custom low-level SDO/PDO passthrough (optional escape hatch)
    // RawControlWord(u16),
}

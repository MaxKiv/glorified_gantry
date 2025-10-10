// Commands that can be sent to the motor
#[derive(Debug, Clone)]
pub enum MotorCommand {
    /// Set continuous velocity
    Home,

    /// Move to an absolute position (in device units, e.g. encoder ticks)
    MoveAbsolute { target: i32, profile_velocity: u32 },

    /// Move relative to current position
    MoveRelative { delta: i32, profile_velocity: u32 },

    /// Set continuous velocity
    SetVelocity { target_velocity: i32 },

    /// Set continuous velocity
    SetTorque { target_torque: i16 },

    /// Halt immediately (stop but remain enabled)
    Halt,

    /// Perform a fault reset sequence
    ResetFault,

    /// Stop motion but keep power enabled.
    // QuickStop,

    /// Disable drive (turn off power stage)
    Disable,

    /// Enable drive (transition to operation enabled)
    Enable,
    // /// Custom low-level SDO/PDO passthrough (optional escape hatch)
    // RawControlWord(u16),
}

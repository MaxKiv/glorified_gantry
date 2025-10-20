// Commands that can be sent to gantry
#[derive(Debug, Clone)]
pub enum GantryCommand {
    /// Home all drives -> Should I even expose this or keep it internal? I like keeping internal maybe
    Home,

    /// Move to an absolute position (in device units, e.g. encoder ticks)
    MoveAbsolute { target: i32, profile_velocity: u32 },

    /// Move relative to current position
    MoveRelative { delta: i32, profile_velocity: u32 },

    /// Set continuous velocity
    SetVelocity { target_velocity: i32 },

    /// Set continuous velocity
    SetTorque { target_torque: i16 },
}

use crate::driver::{cyclic::CyclicSynchronousMode, state::Cia402State};

// Commands that can be sent to the motor
#[derive(Debug, Clone)]
pub enum MotorCommand {
    /// Home this motor
    Home,

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

    /// Transition into target Cia402 State
    Cia402TransitionTo { target_state: Cia402State },

    /// Move to an absolute position (in device units, e.g. encoder ticks)
    /// Note: Only valid when NOT in a Cyclic Synchronous Mode
    MoveAbsolute { target: i32, profile_velocity: u32 },

    /// Move relative to current position
    /// Note: Only valid when NOT in a Cyclic Synchronous Mode
    MoveRelative { delta: i32, profile_velocity: u32 },

    /// Set continuous velocity
    /// Note: Only valid when NOT in a Cyclic Synchronous Mode
    SetVelocity { target_velocity: i32 },

    /// Set continuous velocity
    /// Note: Only valid when NOT in a Cyclic Synchronous Mode
    SetTorque { target_torque: i16 },

    /// Switch into Cyclic Synchronous Mode
    /// Upon switching the driver will reconfigure its T/RPDO mapping to a minimal OnSync set
    /// It will start expecting a continous SYNC and start sending the latest
    /// Cyclic Synchronous Mode target using RPDO on every SYNC cycle
    /// Note: The SYNC cycle generation is left to the user!
    EnterCyclicSynchronousMode { mode: CyclicSynchronousMode },

    /// Exit Cyclic Synchronous Mode
    /// The Driver will reconfigure its T/RPDO mapping to a more verbose OnChange set
    /// and stop responding to SYNC messages
    /// Mode target using RPDO every SYNC cycle
    ExitCyclicSynchronousMode,

    /// Cyclic synchronous position mode update
    /// Note: Only valid when in Cyclic Synchronous Position mode AND the drive is "InSync"
    CyclicSynchronousPosition {
        abs_target: i32,
        // target_velocity: Option<i32>,
        // target_torque: Option<i16>,
    },

    /// Cyclic synchronous velocity mode update
    CyclicSynchronousVelocity {
        target: i32,
        // target_torque: Option<i16>,
    },

    /// Cyclic synchronous torque mode update
    CyclicSynchronousTorque { target: i16 },
}

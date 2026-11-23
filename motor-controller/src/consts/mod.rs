pub mod cfg;
pub mod params;
pub mod pdo;

use std::time::Duration;

use crate::motor::limits::Cia402MotorLimits;

pub const MAX_NODE_ID: usize = 4;

pub struct RtConfig {
    /// RT Thread SYNC cycle period
    pub cycle_period: Duration,

    /// RT Thread max feedback period
    pub feedback_period: Duration,

    /// Tokio -> RT thread cmd queue size
    /// No more consecutive commands can be handled
    pub cmd_channel_size: usize,

    /// RT <-> async command queue size
    /// No more consecutive commands can be handled
    pub cmd_queue_size: usize,

    /// RT <-> async event queue size
    /// No more consecutive events can be observed
    pub event_channel_size: usize,

    /// TODO: determine this, although it seems can.read_frame()
    /// always returns io::error::WouldBlock after 1 call when
    /// CAN.set_nonblocking(true)
    pub can_frames_per_poll: usize,

    /// Maximum tolerated axis skew
    pub max_axis_skew_mm: f64,
}

// --- Tunable constants ---
pub const RT_CONFIG: RtConfig = RtConfig {
    cycle_period: Duration::from_millis(500),
    feedback_period: Duration::from_micros(250),
    cmd_queue_size: 64,
    cmd_channel_size: 8,
    event_channel_size: 64,
    can_frames_per_poll: 1,
    max_axis_skew_mm: 1.0,
};

pub const DEMO_LIMITS: Cia402MotorLimits = Cia402MotorLimits {
    profile_velocity: 30u32,       // Revs/minute
    profile_acceleration: 20_000,  // Counts/s^2
    profile_decceleration: 20_000, // Counts/s^2
};

pub const TEST_LIMITS: Cia402MotorLimits = Cia402MotorLimits {
    profile_velocity: 15u32,       // Revs/minute
    profile_acceleration: 20_000,  // Counts/s^2
    profile_decceleration: 20_000, // Counts/s^2
};

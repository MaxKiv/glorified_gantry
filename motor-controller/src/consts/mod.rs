pub mod pdo;

use std::time::Duration;

pub struct RtConfig {
    /// RT Thread SYNC cycle period
    pub cycle_period: Duration,

    /// RT Thread max feedback period
    pub feedback_period: Duration,

    /// Tokio -> RT thread cmd queue size
    /// No more consecutive commands can be handled
    pub cmd_channel_size: usize,

    /// RT internal command queue size
    /// No more consecutive commands can be handled
    pub cmd_queue_size: usize,

    /// TODO: determine this, although it seems can.read_frame()
    /// always returns io::error::WouldBlock after 1 call when
    /// CAN.set_nonblocking(true)
    pub can_frames_per_poll: usize,
}

// --- Tunable constants ---
pub const RT_CONFIG: RtConfig = RtConfig {
    cycle_period: Duration::from_millis(500),
    feedback_period: Duration::from_micros(250),
    cmd_queue_size: 64,
    // cmd_channel_size: 64,
    cmd_channel_size: 8,
    can_frames_per_poll: 1,
};

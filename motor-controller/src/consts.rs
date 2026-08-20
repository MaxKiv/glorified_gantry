use std::time::Duration;

// --- Tunable constants ---
/// RT Thread SYNC cycle period
pub const RT_CYCLE_PERIOD: Duration = Duration::from_millis(500);
/// TODO: determine this, although it seems can.read_frame()
/// always returns io::error::WouldBlock after 1 call when
/// CAN.set_nonblocking(true)
pub const MAX_CAN_RX_PER_POLL: usize = 1;

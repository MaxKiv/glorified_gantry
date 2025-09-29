use crate::od::{drive_subscriber::StatusWord, state::Cia402State};

/// Events broadcast by a motor driver (status updates, transitions, errors).
#[derive(Debug, Clone)]
pub enum MotorEvent {
    /// State machine transition (Not Ready -> Ready to Switch On, etc.)
    StateChanged { old: Cia402State, new: Cia402State },

    /// New statusword received from device
    StatusWord(StatusWord),

    /// Position feedback (encoder units or user-scaled units)
    PositionFeedback { actual_position: i32 },

    /// Velocity feedback
    VelocityFeedback { actual_velocity: i32 },

    /// Torque feedback
    TorqueFeedback { actual_torque: i32 },

    /// Fault detected (e.g. fault bit set in statusword)
    Fault { code: u16, description: String },

    /// Drive recovered from fault
    FaultCleared,

    /// Communication to Drive lost
    CommunicationLost,

    /// Generic async notification (homing completed, target reached, etc.)
    Notification(String),
}

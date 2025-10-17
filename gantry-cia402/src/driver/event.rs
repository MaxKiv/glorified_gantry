use crate::driver::{
    nmt::NmtState,
    oms::OperationMode,
    receiver::{
        StatusWord,
        parse::{self, sdo_response::SdoResponse},
    },
    state::Cia402State,
};

/// Events broadcast by a motor driver (status updates, transitions, errors).
#[derive(Debug, Clone, PartialEq)]
pub enum MotorEvent {
    /// NMT state update
    Cia402StateUpdate(Cia402State),

    /// NMT state update
    NmtStateUpdate(NmtState),

    /// Operational mode update
    OperationModeUpdate(OperationMode),

    /// New statusword received from device
    StatusWord(StatusWord),

    /// Position feedback [counts]
    PositionFeedback { actual_position: i32 },

    /// Velocity feedback [counts/min]
    VelocityFeedback { actual_velocity: i32 },

    /// Torque feedback
    TorqueFeedback { actual_torque: i16 },

    /// Homing feedback
    /// Simplification of datasheet page 71
    HomingFeedback {
        at_home: bool,
        homing_completed: bool,
        homing_error: bool,
    },

    /// Position mode feedback
    PositionModeFeedback {
        target_reached: bool,
        limit_exceeded: bool,
        setpoint_acknowlegded: bool,
        following_error: bool,
    },

    /// Velocity mode feedback
    VelocityModeFeedback {
        speed_is_zero: bool,
        deviation_error: bool,
    },

    /// Torque mode feedback
    TorqueModeFeedback { limit_exceeded: bool },

    /// Fault detected (e.g. fault bit set in statusword)
    Fault { code: u16, description: String },

    /// EMCY message from motor driver
    EMCY(parse::EMCY),

    /// SDO response received
    SdoResponse(SdoResponse),

    /// Drive recovered from fault
    FaultCleared,

    /// Communication to Drive lost
    CommunicationLost,
}

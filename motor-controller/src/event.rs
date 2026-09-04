use crate::{cia402::Cia402State, oms::OperationMode, sw::StatusWord};

/// Events broadcast by a motor driver (status updates, transitions, errors).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotorEvent {
    /// NMT state update
    Cia402StateUpdate(Cia402State),

    /// NMT state update
    // NmtStateUpdate(NmtState),

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
    TorqueModeFeedback {
        axis_braked: bool,
        setpoint_reached: bool,
        limit_exceeded: bool,
    },

    /// Cyclic Position mode feedback
    CyclicPositionModeFeedback {
        device_in_sync: bool,
        is_following_target: bool,
        has_following_error: bool,
    },

    /// Cyclic Velocity mode feedback
    CyclicVelocityModeFeedback {
        device_in_sync: bool,
        is_following_target: bool,
    },

    /// Cyclic Torque mode feedback
    CyclicTorqueModeFeedback {
        device_in_sync: bool,
        is_following_target: bool,
    },

    /// Fault detected (e.g. fault bit set in statusword)
    Fault { code: u16 },

    /// EMCY message from motor driver
    // EMCY(parse::EMCY),

    /// SDO response received
    // SdoResponse(SdoResponse),

    /// Drive recovered from fault
    FaultCleared,

    /// Communication to Drive lost
    CommunicationLost,
}

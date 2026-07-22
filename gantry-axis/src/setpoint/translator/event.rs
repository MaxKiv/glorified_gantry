use gantry_cia402::driver::{
    nmt::NmtState,
    receiver::{
        StatusWord,
        parse::{self, sdo_response::SdoResponse},
    },
    state::Cia402State,
};
use uom::si::f64::{Length, Torque, Velocity};

use crate::OperationMode;

/// Events broadcast by a motor driver (status updates, transitions, errors).
#[derive(Debug, Clone, PartialEq)]
pub enum TranslatedMotorEvent {
    /// NMT state update
    Cia402StateUpdate(Cia402State),

    /// NMT state update
    NmtStateUpdate(NmtState),

    /// Operational mode update
    OperationModeUpdate(OperationMode),

    /// New statusword received from device
    StatusWord(StatusWord),

    /// Translated Position feedback
    /// Note: i32 translated into Length quantity
    PositionFeedback { actual_position: Length },

    /// Translated Velocity feedback
    /// Note: i32 translated into Velocity quantity
    VelocityFeedback { actual_velocity: Velocity },

    /// Translated Torque feedback
    /// Note: i16 translated into Length quantity
    TorqueFeedback { actual_torque: Torque },

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
    EMCY(parse::EMCY),

    /// SDO response received
    SdoResponse(SdoResponse),

    /// Drive recovered from fault
    FaultCleared,

    /// Communication to Drive lost
    CommunicationLost,
}

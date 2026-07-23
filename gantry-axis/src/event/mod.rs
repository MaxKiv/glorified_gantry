use gantry_cia402::driver::receiver::parse::EMCY;
use oze_canopen::canopen::NodeId;

use crate::{
    OperationMode,
    axis::Axis,
    axis_state::AxisState,
    diagnostic::{DiagnosticContent, DiagnosticLevel},
};

pub mod combiner;
pub mod from_motor;
pub mod handler;
pub mod types;
pub mod util;

/// Motor Specific events emitted by the gantry driver
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GantryMotorEvent {
    pub motor: NodeId,
    pub axis: Axis,
    pub content: GantryMotorEventContent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GantryMotorEventContent {
    /// Position update
    Position { value: f64 },

    /// Motor Position mode specific feedback
    PositionModeFeedback {
        target_reached: bool,
        limit_exceeded: bool,
        setpoint_acknowlegded: bool,
        following_error: bool,
    },

    /// Velocity update
    Velocity { value: f64 },

    /// Velocity mode feedback
    VelocityModeFeedback {
        speed_is_zero: bool,
        deviation_error: bool,
    },

    /// Torque feedback or effort
    Torque { value: f64 },

    /// Torque mode feedback
    TorqueModeFeedback {
        axis_braked: bool,
        setpoint_reached: bool,
        limit_exceeded: bool,
    },

    /// Operation mode change
    ModeChanged { mode: OperationMode },

    /// Axis status/state update (cia402 + NMT merged)
    AxisState { state: AxisState },

    /// Homing progress or completion
    Homing {
        at_home: bool,
        completed: bool,
        error: bool,
    },

    /// Fault or error
    Fault { code: u16 },

    /// Fault or error
    Emcy { emcy: EMCY },

    /// Informational / diagnostic message
    Diagnostic {
        level: DiagnosticLevel,
        content: DiagnosticContent,
    },

    /// Communication loss, recovered, or sync feedback
    SyncStatus {
        in_sync: bool,
        following_error: bool,
    },
}

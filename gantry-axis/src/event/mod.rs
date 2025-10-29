use gantry_cia402::driver::{oms::OperationMode, receiver::parse::EMCY};

use crate::{axis::Axis, axis_state::AxisState, diagnostic::DiagnosticLevel};

pub mod combiner;
pub mod from_motor;
pub mod handler;
pub mod types;
pub mod util;

#[derive(Debug, Clone, PartialEq)]
pub enum GantryEvent {
    /// Position update in physical units (m, rad, etc.)
    Position { axis: Axis, value: f64 },

    /// Motor Position mode specific feedback
    PositionModeFeedback {
        axis: Axis,
        target_reached: bool,
        limit_exceeded: bool,
        setpoint_acknowlegded: bool,
        following_error: bool,
    },

    /// Velocity update
    Velocity { axis: Axis, value: f64 },

    /// Velocity mode feedback
    VelocityModeFeedback {
        axis: Axis,
        speed_is_zero: bool,
        deviation_error: bool,
    },

    /// Torque feedback or effort
    Torque { axis: Axis, value: f64 },

    /// Torque mode feedback
    TorqueModeFeedback {
        axis: Axis,
        axis_braked: bool,
        setpoint_reached: bool,
        limit_exceeded: bool,
    },

    /// Operation mode change
    ModeChanged { axis: Axis, mode: OperationMode },

    /// Axis status/state update (cia402 + NMT merged)
    AxisState { axis: Axis, state: AxisState },

    /// Homing progress or completion
    Homing {
        axis: Axis,
        at_home: bool,
        completed: bool,
        error: bool,
    },

    /// Fault or error
    Fault {
        axis: Axis,
        code: u16,
        description: String,
    },

    /// Fault or error
    Emcy { axis: Axis, emcy: EMCY },

    /// Informational / diagnostic message
    Diagnostic {
        axis: Axis,
        level: DiagnosticLevel,
        message: String,
    },

    /// Communication loss, recovered, or sync feedback
    SyncStatus {
        axis: Axis,
        in_sync: bool,
        following_error: bool,
    },
}

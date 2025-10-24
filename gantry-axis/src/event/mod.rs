use gantry_cia402::driver::{oms::OperationMode, receiver::parse::EMCY};

use crate::{axis::Axis, axis_state::AxisState, diagnostic::DiagnosticLevel};

pub mod from_motor;
pub mod handler;
pub mod util;

#[derive(Debug, Clone, PartialEq)]
pub enum GantryEvent {
    /// Position update in physical units (m, rad, etc.)
    Position { axis: Axis, value: f64 },

    /// Velocity update
    Velocity { axis: Axis, value: f64 },

    /// Torque feedback or effort
    Torque { axis: Axis, value: f64 },

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

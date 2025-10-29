use uom::si::{length::millimeter, torque::newton_meter, velocity::meter_per_second};

use crate::{
    axis::Axis, axis_state::AxisState, diagnostic::DiagnosticLevel, event::GantryEvent,
    setpoint::translator::event::TranslatedMotorEvent,
};

impl GantryEvent {
    pub fn from_translated(axis: Axis, event: TranslatedMotorEvent) -> Self {
        use TranslatedMotorEvent::*;
        match event {
            PositionFeedback { actual_position } => GantryEvent::Position {
                axis,
                value: actual_position.get::<millimeter>(),
            },
            VelocityFeedback { actual_velocity } => GantryEvent::Velocity {
                axis,
                value: actual_velocity.get::<meter_per_second>(),
            },
            TorqueFeedback { actual_torque } => GantryEvent::Torque {
                axis,
                value: actual_torque.get::<newton_meter>(),
            },

            OperationModeUpdate(mode) => GantryEvent::ModeChanged { axis, mode },

            Cia402StateUpdate(state) => GantryEvent::AxisState {
                axis,
                state: AxisState::Cia402(state),
            },
            NmtStateUpdate(state) => GantryEvent::AxisState {
                axis,
                state: AxisState::Nmt(state),
            },

            HomingFeedback {
                at_home,
                homing_completed,
                homing_error,
            } => GantryEvent::Homing {
                axis,
                at_home,
                completed: homing_completed,
                error: homing_error,
            },

            PositionModeFeedback {
                target_reached,
                limit_exceeded,
                setpoint_acknowlegded,
                following_error,
            } => GantryEvent::PositionModeFeedback {
                axis,
                target_reached,
                limit_exceeded,
                setpoint_acknowlegded,
                following_error,
            },

            VelocityModeFeedback {
                speed_is_zero,
                deviation_error,
            } => GantryEvent::VelocityModeFeedback {
                axis,
                speed_is_zero,
                deviation_error,
            },

            TorqueModeFeedback {
                axis_braked,
                setpoint_reached,
                limit_exceeded,
            } => GantryEvent::TorqueModeFeedback {
                axis,
                axis_braked,
                setpoint_reached,
                limit_exceeded,
            },

            CyclicPositionModeFeedback {
                device_in_sync,
                has_following_error,
                ..
            } => GantryEvent::SyncStatus {
                axis,
                in_sync: device_in_sync,
                following_error: has_following_error,
            },
            CyclicVelocityModeFeedback { device_in_sync, .. } => GantryEvent::SyncStatus {
                axis,
                in_sync: device_in_sync,
                following_error: false,
            },
            CyclicTorqueModeFeedback { device_in_sync, .. } => GantryEvent::SyncStatus {
                axis,
                in_sync: device_in_sync,
                following_error: false,
            },

            Fault { code, description } => GantryEvent::Fault {
                axis,
                code,
                description,
            },
            EMCY(emcy) => GantryEvent::Emcy { axis, emcy },
            FaultCleared => GantryEvent::Diagnostic {
                axis,
                level: DiagnosticLevel::Ok,
                message: "Fault cleared".into(),
            },
            CommunicationLost => GantryEvent::Diagnostic {
                axis,
                level: DiagnosticLevel::Error,
                message: "Communication lost".into(),
            },
            SdoResponse(resp) => GantryEvent::Diagnostic {
                axis,
                level: DiagnosticLevel::Ok,
                message: format!("SDO: {:?}", resp),
            },
            StatusWord(_) => GantryEvent::Diagnostic {
                axis,
                level: DiagnosticLevel::Ok,
                message: "Statusword update".into(),
            },
        }
    }
}

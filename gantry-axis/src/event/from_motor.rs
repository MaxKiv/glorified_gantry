use gantry_cia402::driver::event::MotorEvent;

use crate::{
    axis::Axis, axis_state::AxisState, diagnostic::DiagnosticLevel, event::GantryEvent,
    setpoint::translator::SetpointTranslator,
};

impl GantryEvent {
    pub fn from_motor(axis: Axis, event: MotorEvent, translator: &SetpointTranslator) -> Self {
        match event {
            MotorEvent::PositionFeedback { actual_position } => GantryEvent::Position {
                axis,
                value: actual_position as f64,
            },
            MotorEvent::VelocityFeedback { actual_velocity } => GantryEvent::Velocity {
                axis,
                value: actual_velocity as f64,
            },
            MotorEvent::TorqueFeedback { actual_torque } => GantryEvent::Torque {
                axis,
                value: actual_torque as f64,
            },

            MotorEvent::OperationModeUpdate(mode) => GantryEvent::ModeChanged { axis, mode },

            MotorEvent::Cia402StateUpdate(state) => GantryEvent::AxisState {
                axis,
                state: AxisState::Cia402(state),
            },
            MotorEvent::NmtStateUpdate(state) => GantryEvent::AxisState {
                axis,
                state: AxisState::Nmt(state),
            },

            MotorEvent::HomingFeedback {
                at_home,
                homing_completed,
                homing_error,
            } => GantryEvent::Homing {
                axis,
                at_home,
                completed: homing_completed,
                error: homing_error,
            },

            MotorEvent::PositionModeFeedback {
                target_reached,
                following_error,
                ..
            } => GantryEvent::SyncStatus {
                axis,
                in_sync: target_reached,
                following_error,
            },

            MotorEvent::VelocityModeFeedback {
                deviation_error, ..
            } => GantryEvent::Diagnostic {
                axis,
                level: if deviation_error {
                    DiagnosticLevel::Error
                } else {
                    DiagnosticLevel::Ok
                },
                message: "Velocity feedback".into(),
            },

            MotorEvent::TorqueModeFeedback {
                setpoint_reached,
                limit_exceeded,
                ..
            } => GantryEvent::Diagnostic {
                axis,
                level: if limit_exceeded {
                    DiagnosticLevel::Warn
                } else {
                    DiagnosticLevel::Ok
                },
                message: format!("Torque setpoint reached: {}", setpoint_reached),
            },

            MotorEvent::CyclicPositionModeFeedback {
                device_in_sync,
                has_following_error,
                ..
            } => GantryEvent::SyncStatus {
                axis,
                in_sync: device_in_sync,
                following_error: has_following_error,
            },
            MotorEvent::CyclicVelocityModeFeedback { device_in_sync, .. } => {
                GantryEvent::SyncStatus {
                    axis,
                    in_sync: device_in_sync,
                    following_error: false,
                }
            }
            MotorEvent::CyclicTorqueModeFeedback { device_in_sync, .. } => {
                GantryEvent::SyncStatus {
                    axis,
                    in_sync: device_in_sync,
                    following_error: false,
                }
            }

            MotorEvent::Fault { code, description } => GantryEvent::Fault {
                axis,
                code,
                description,
            },
            MotorEvent::EMCY(emcy) => GantryEvent::Emcy { axis, emcy },
            MotorEvent::FaultCleared => GantryEvent::Diagnostic {
                axis,
                level: DiagnosticLevel::Ok,
                message: "Fault cleared".into(),
            },
            MotorEvent::CommunicationLost => GantryEvent::Diagnostic {
                axis,
                level: DiagnosticLevel::Error,
                message: "Communication lost".into(),
            },
            MotorEvent::SdoResponse(resp) => GantryEvent::Diagnostic {
                axis,
                level: DiagnosticLevel::Ok,
                message: format!("SDO: {:?}", resp),
            },
            MotorEvent::StatusWord(_) => GantryEvent::Diagnostic {
                axis,
                level: DiagnosticLevel::Ok,
                message: "Statusword update".into(),
            },
        }
    }
}

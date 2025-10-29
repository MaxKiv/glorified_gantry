use gantry_cia402::driver::event::MotorEvent;
use uom::si::{length::millimeter, torque::newton_meter, velocity::meter_per_second};

use crate::{
    axis::Axis, axis_state::AxisState, diagnostic::DiagnosticLevel, event::GantryEvent,
    setpoint::translator::SetpointTranslator,
};

impl GantryEvent {
    pub fn from_motor(axis: Axis, event: MotorEvent, translator: &SetpointTranslator) -> Self {
        match event {
            MotorEvent::PositionFeedback { actual_position } => GantryEvent::Position {
                axis,
                value: translator
                    .translate_motor_position(actual_position)
                    .get::<millimeter>(),
            },
            MotorEvent::VelocityFeedback { actual_velocity } => GantryEvent::Velocity {
                axis,
                value: translator
                    .translate_motor_velocity(actual_velocity)
                    .get::<meter_per_second>(),
            },
            MotorEvent::TorqueFeedback { actual_torque } => GantryEvent::Torque {
                axis,
                value: translator
                    .translate_motor_torque(actual_torque)
                    .get::<newton_meter>(),
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

            MotorEvent::VelocityModeFeedback {
                speed_is_zero,
                deviation_error,
            } => GantryEvent::VelocityModeFeedback {
                axis,
                speed_is_zero,
                deviation_error,
            },

            MotorEvent::TorqueModeFeedback {
                axis_braked,
                setpoint_reached,
                limit_exceeded,
            } => GantryEvent::TorqueModeFeedback {
                axis,
                axis_braked,
                setpoint_reached,
                limit_exceeded,
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

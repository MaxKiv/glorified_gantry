use oze_canopen::canopen::NodeId;
use uom::si::{length::millimeter, torque::newton_meter, velocity::meter_per_second};

use crate::{
    axis::Axis,
    axis_state::AxisState,
    diagnostic::{DiagnosticContent, DiagnosticLevel},
    event::{GantryMotorEvent, GantryMotorEventContent},
    setpoint::translator::event::TranslatedMotorEvent,
};

impl GantryMotorEvent {
    pub fn from_translated(from_id: NodeId, axis: Axis, event: TranslatedMotorEvent) -> Self {
        use TranslatedMotorEvent::*;
        match event {
            PositionFeedback { actual_position } => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::Position {
                    value: actual_position.get::<millimeter>(),
                },
            },
            VelocityFeedback { actual_velocity } => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::Velocity {
                    value: actual_velocity.get::<meter_per_second>(),
                },
            },
            TorqueFeedback { actual_torque } => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::Torque {
                    value: actual_torque.get::<newton_meter>(),
                },
            },

            OperationModeUpdate(mode) => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::ModeChanged { mode },
            },

            Cia402StateUpdate(state) => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::AxisState {
                    state: AxisState::Cia402(state),
                },
            },
            NmtStateUpdate(state) => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::AxisState {
                    state: AxisState::Nmt(state),
                },
            },

            HomingFeedback {
                at_home,
                homing_completed,
                homing_error,
            } => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::Homing {
                    at_home,
                    completed: homing_completed,
                    error: homing_error,
                },
            },

            PositionModeFeedback {
                target_reached,
                limit_exceeded,
                setpoint_acknowlegded,
                following_error,
            } => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::PositionModeFeedback {
                    target_reached,
                    limit_exceeded,
                    setpoint_acknowlegded,
                    following_error,
                },
            },

            VelocityModeFeedback {
                speed_is_zero,
                deviation_error,
            } => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::VelocityModeFeedback {
                    speed_is_zero,
                    deviation_error,
                },
            },

            TorqueModeFeedback {
                axis_braked,
                setpoint_reached,
                limit_exceeded,
            } => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::TorqueModeFeedback {
                    axis_braked,
                    setpoint_reached,
                    limit_exceeded,
                },
            },

            CyclicPositionModeFeedback {
                device_in_sync,
                has_following_error,
                ..
            } => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::SyncStatus {
                    in_sync: device_in_sync,
                    following_error: has_following_error,
                },
            },
            CyclicVelocityModeFeedback { device_in_sync, .. } => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::SyncStatus {
                    in_sync: device_in_sync,
                    following_error: false,
                },
            },
            CyclicTorqueModeFeedback { device_in_sync, .. } => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::SyncStatus {
                    in_sync: device_in_sync,
                    following_error: false,
                },
            },

            Fault { code } => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::Fault { code },
            },
            EMCY(emcy) => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::Emcy { emcy },
            },
            FaultCleared => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::Diagnostic {
                    level: DiagnosticLevel::Ok,
                    content: DiagnosticContent::FaultCleared,
                },
            },
            CommunicationLost => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::Diagnostic {
                    level: DiagnosticLevel::Error,
                    content: DiagnosticContent::CommunicationLost,
                },
            },
            SdoResponse(resp) => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::Diagnostic {
                    level: DiagnosticLevel::Ok,
                    content: DiagnosticContent::SdoResponse(resp),
                },
            },
            StatusWord(sw) => GantryMotorEvent {
                motor: from_id,
                axis,
                content: GantryMotorEventContent::Diagnostic {
                    level: DiagnosticLevel::Ok,
                    content: DiagnosticContent::StatusWordUpdate(sw),
                },
            },
        }
    }
}

pub mod event;
pub mod scaling;

use gantry_cia402::driver::{command::MotorCommand, event::MotorEvent};
use tracing::info;
use uom::si::f64::Velocity;
use uom::si::f64::{Length, Torque};

use crate::axis::setpoint::*;
use crate::setpoint::translator::event::TranslatedMotorEvent;
use crate::{event::GantryEvent, setpoint::translator::scaling::DeviceScaling};

#[derive(Debug, Clone)]
pub struct SetpointTranslator {
    scaling: DeviceScaling,
}

impl SetpointTranslator {
    pub fn new(scaling: &DeviceScaling) -> Self {
        Self {
            scaling: scaling.clone(),
        }
    }

    pub fn to_motor_cmd(&self, setpoint: AxisSetpoint) -> MotorCommand {
        let cmd = match setpoint.clone() {
            AxisSetpoint::RelativePosition(position_setpoint) => {
                // Scale setpoint
                let delta = self.scaling.to_device_pos(position_setpoint.target);
                let profile_velocity = self.scaling.to_device_abs_vel(position_setpoint.velocity);

                MotorCommand::MoveRelative {
                    delta,
                    profile_velocity,
                }
            }
            AxisSetpoint::AbsolutePosition(position_setpoint) => {
                // Scale setpoint
                let target = self.scaling.to_device_pos(position_setpoint.target);
                let profile_velocity = self.scaling.to_device_abs_vel(position_setpoint.velocity);

                MotorCommand::MoveAbsolute {
                    target,
                    profile_velocity,
                }
            }
            AxisSetpoint::Velocity(velocity_setpoint) => {
                // Scale setpoint
                let target_velocity = self.scaling.to_device_vel(velocity_setpoint.target);

                MotorCommand::SetVelocity { target_velocity }
            }
            AxisSetpoint::Torque(torque_setpoint) => {
                // Scale setpoint
                let target_torque = self.scaling.to_device_torque(torque_setpoint.target);

                MotorCommand::SetTorque { target_torque }
            }
        };

        info!("Gantry translated {0:?} into {1:?}", setpoint, cmd);

        cmd
    }

    /// Translates motor units in given MotorEvents into SI units
    pub fn translate_motor_event(&self, event: MotorEvent) -> TranslatedMotorEvent {
        use MotorEvent::*;
        match event {
            // Translate motor units -> SI units for feedback events
            PositionFeedback { actual_position } => TranslatedMotorEvent::PositionFeedback {
                actual_position: self.translate_motor_position(actual_position),
            },
            VelocityFeedback { actual_velocity } => TranslatedMotorEvent::VelocityFeedback {
                actual_velocity: self.translate_motor_velocity(actual_velocity),
            },
            TorqueFeedback { actual_torque } => TranslatedMotorEvent::TorqueFeedback {
                actual_torque: self.translate_motor_torque(actual_torque),
            },

            // Identity transform for remaining events
            Cia402StateUpdate(cia402_state) => {
                TranslatedMotorEvent::Cia402StateUpdate(cia402_state)
            }
            NmtStateUpdate(nmt_state) => TranslatedMotorEvent::NmtStateUpdate(nmt_state),
            OperationModeUpdate(operation_mode) => {
                TranslatedMotorEvent::OperationModeUpdate(operation_mode)
            }
            StatusWord(status_word) => TranslatedMotorEvent::StatusWord(status_word),
            HomingFeedback {
                at_home,
                homing_completed,
                homing_error,
            } => TranslatedMotorEvent::HomingFeedback {
                at_home,
                homing_completed,
                homing_error,
            },
            PositionModeFeedback {
                target_reached,
                limit_exceeded,
                setpoint_acknowlegded,
                following_error,
            } => TranslatedMotorEvent::PositionModeFeedback {
                target_reached,
                limit_exceeded,
                setpoint_acknowlegded,
                following_error,
            },
            VelocityModeFeedback {
                speed_is_zero,
                deviation_error,
            } => TranslatedMotorEvent::VelocityModeFeedback {
                speed_is_zero,
                deviation_error,
            },
            TorqueModeFeedback {
                axis_braked,
                setpoint_reached,
                limit_exceeded,
            } => TranslatedMotorEvent::TorqueModeFeedback {
                axis_braked,
                setpoint_reached,
                limit_exceeded,
            },
            CyclicPositionModeFeedback {
                device_in_sync,
                is_following_target,
                has_following_error,
            } => TranslatedMotorEvent::CyclicPositionModeFeedback {
                device_in_sync,
                is_following_target,
                has_following_error,
            },
            CyclicVelocityModeFeedback {
                device_in_sync,
                is_following_target,
            } => TranslatedMotorEvent::CyclicVelocityModeFeedback {
                device_in_sync,
                is_following_target,
            },
            CyclicTorqueModeFeedback {
                device_in_sync,
                is_following_target,
            } => TranslatedMotorEvent::CyclicTorqueModeFeedback {
                device_in_sync,
                is_following_target,
            },
            Fault { code, description } => TranslatedMotorEvent::Fault { code, description },
            EMCY(emcy) => TranslatedMotorEvent::EMCY(emcy),
            SdoResponse(sdo_response) => TranslatedMotorEvent::SdoResponse(sdo_response),
            FaultCleared => TranslatedMotorEvent::FaultCleared,
            CommunicationLost => TranslatedMotorEvent::CommunicationLost,
        }
    }

    pub fn translate_motor_position(&self, pos: i32) -> Length {
        self.scaling.from_device_pos(pos)
    }

    pub fn translate_motor_velocity(&self, vel: i32) -> Velocity {
        self.scaling.from_device_vel(vel)
    }

    pub fn translate_motor_absolute_velocity(&self, abs_vel: u32) -> Velocity {
        self.scaling.from_device_abs_vel(abs_vel)
    }

    pub fn translate_motor_torque(&self, torque: i16) -> Torque {
        self.scaling.from_device_torque(torque)
    }
}

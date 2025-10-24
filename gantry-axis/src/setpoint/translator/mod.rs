pub mod scaling;

use gantry_cia402::driver::{command::MotorCommand, event::MotorEvent};
use tracing::info;
use uom::si::f64::Velocity;
use uom::si::f64::{Length, Torque};

use crate::axis::setpoint::*;
use crate::{event::GantryEvent, setpoint::translator::scaling::DeviceScaling};

#[derive(Debug, Clone)]
pub struct SetpointTranslator {
    scaling: DeviceScaling,
}

impl SetpointTranslator {
    pub fn new(scaling: DeviceScaling) -> Self {
        Self { scaling }
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

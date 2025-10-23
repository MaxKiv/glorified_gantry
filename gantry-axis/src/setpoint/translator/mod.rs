pub mod scaling;

use gantry_cia402::driver::{command::MotorCommand, event::MotorEvent};
use tracing::info;

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
                let delta = self.scaling.position(position_setpoint.target);
                let profile_velocity = self.scaling.abs_velocity(position_setpoint.velocity);

                MotorCommand::MoveRelative {
                    delta,
                    profile_velocity,
                }
            }
            AxisSetpoint::AbsolutePosition(position_setpoint) => {
                // Scale setpoint
                let target = self.scaling.position(position_setpoint.target);
                let profile_velocity = self.scaling.abs_velocity(position_setpoint.velocity);

                MotorCommand::MoveAbsolute {
                    target,
                    profile_velocity,
                }
            }
            AxisSetpoint::Velocity(velocity_setpoint) => {
                // Scale setpoint
                let target_velocity = self.scaling.velocity(velocity_setpoint.target);

                MotorCommand::SetVelocity { target_velocity }
            }
            AxisSetpoint::Torque(torque_setpoint) => {
                // Scale setpoint
                let target_torque = self.scaling.torque(torque_setpoint.target);

                MotorCommand::SetTorque { target_torque }
            }
        };

        info!("Gantry translated {0:?} into {1:?}", setpoint, cmd);

        cmd
    }

    pub fn from_motor_event(event: MotorEvent) -> GantryEvent {
        todo!()
    }
}

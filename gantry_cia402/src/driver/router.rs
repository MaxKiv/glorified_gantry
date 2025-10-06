use crate::{
    driver::oms::{
        PositionModeFlags, PositionSetpoint, Setpoint, TorqueSetpoint, VelocitySetpoint,
    },
    driver::{command::MotorCommand, state::Cia402State},
};

use tokio::sync::mpsc;
use tracing::*;

pub async fn command_router(
    mut cmd_rx: mpsc::Receiver<MotorCommand>,
    state_cmd_tx: tokio::sync::mpsc::Sender<Cia402State>,
    setpoint_cmd_tx: tokio::sync::mpsc::Sender<Setpoint>,
) {
    if let Some(cmd) = cmd_rx.recv().await {
        trace!(
            "Router received Command: {cmd:?}, routing to the cia402 state machine or operation mode specific handler"
        );

        match cmd {
            MotorCommand::Halt => {
                const SAFE_POS: i32 = 0;
                const SAFE_VEL: u32 = 0;

                setpoint_cmd_tx
                    .send(Setpoint::ProfilePosition(PositionSetpoint {
                        flags: PositionModeFlags::default()
                            | PositionModeFlags::RELATIVE
                            | PositionModeFlags::HALT,
                        target: SAFE_POS,
                        profile_velocity: SAFE_VEL,
                    }))
                    .await;
            }
            MotorCommand::Enable => {
                state_cmd_tx.send(Cia402State::ReadyToSwitchOn).await;
                state_cmd_tx.send(Cia402State::SwitchedOn).await;
                state_cmd_tx.send(Cia402State::OperationEnabled).await;
            }
            MotorCommand::Disable => {
                state_cmd_tx.send(Cia402State::OperationEnabled).await;
                state_cmd_tx.send(Cia402State::SwitchedOn).await;
                state_cmd_tx.send(Cia402State::ReadyToSwitchOn).await;
            }
            MotorCommand::MoveAbsolute {
                target,
                profile_velocity,
            } => {
                setpoint_cmd_tx
                    .send(Setpoint::ProfilePosition(PositionSetpoint {
                        flags: PositionModeFlags::default(),
                        target,
                        profile_velocity,
                    }))
                    .await;
            }
            MotorCommand::MoveRelative {
                delta,
                profile_velocity,
            } => {
                setpoint_cmd_tx
                    .send(Setpoint::ProfilePosition(PositionSetpoint {
                        flags: PositionModeFlags::default() | PositionModeFlags::RELATIVE,
                        target: delta,
                        profile_velocity,
                    }))
                    .await;
            }
            MotorCommand::SetVelocity { target_velocity } => {
                setpoint_cmd_tx
                    .send(Setpoint::ProfileVelocity(VelocitySetpoint {
                        target_velocity,
                    }))
                    .await;
            }
            MotorCommand::SetTorque { target_torque } => {
                setpoint_cmd_tx
                    .send(Setpoint::ProfileTorque(TorqueSetpoint { target_torque }))
                    .await;
            }
        };
    }
}

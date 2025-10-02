use crate::{
    driver::{command::MotorCommand, event::MotorEvent, state::Cia402State},
    od::oms::{
        DEFAULT_POSITIONMODE_FLAGS, PositionModeFlags, PositionSetpoint, Setpoint, TorqueSetpoint,
        VelocitySetpoint,
    },
};

use tokio::sync::mpsc;
use tracing::*;

pub async fn command_router(
    cmd_rx: mpsc::Receiver<MotorCommand>,
    state_cmd_tx: tokio::sync::mpsc::Sender<Cia402State>,
    setpoint_cmd_tx: tokio::sync::mpsc::Sender<Setpoint>,
) {
    if let Some(cmd) = cmd_rx.recv().await {
        trace!(
            "Router received Command: {cmd:?}, routing to the cia402 state machine or operation mode specific handler"
        );
        let update = match cmd {
            MotorCommand::Halt => setpoint_cmd_tx.send(Setpoint::Halt),
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
            } => setpoint_cmd_tx.send(Setpoint::ProfilePosition(PositionSetpoint {
                flags: DEFAULT_POSITIONMODE_FLAGS,
                target,
                profile_velocity,
            })),
            MotorCommand::MoveRelative {
                delta,
                profile_velocity,
            } => setpoint_cmd_tx.send(Setpoint::ProfilePosition(PositionSetpoint {
                flags: DEFAULT_POSITIONMODE_FLAGS |= PositionModeFlags::RELATIVE,
                target: delta,
                profile_velocity,
            })),
            MotorCommand::SetVelocity { target_velocity } => {
                setpoint_cmd_tx.send(Setpoint::ProfileVelocity(VelocitySetpoint {
                    target_velocity,
                }))
            }
            MotorCommand::SetTorque { target_torque } => {
                setpoint_cmd_tx.send(Setpoint::ProfileTorque(TorqueSetpoint { target_torque }))
            }
        }
        .await;
    }
}

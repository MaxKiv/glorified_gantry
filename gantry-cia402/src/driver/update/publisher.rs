use std::sync::Arc;

use tokio::sync::{
    Mutex, broadcast,
    mpsc::{self},
};
use tracing::*;

use crate::{
    comms::pdo::Pdo,
    driver::{
        command::MotorCommand,
        oms::{
            home::{HomeFlagsCW, HomingSetpoint},
            position::{PositionFlagsCW, PositionSetpoint},
            setpoint::Setpoint,
            torque::TorqueSetpoint,
            velocity::VelocitySetpoint,
        },
        receiver::setpoint_manager::SetpointManager,
        state::Cia402Flags,
    },
};

/// Responsible for all CANopen communication to the drive
/// Receives updates from the Cia402 state machine and operational mode specific handler
/// It encodes these changes into the appropriate controlword bits or OD object
/// It then sends these changes out on the CANopen bus using the accessor
pub async fn publish_updates(
    pdo: Arc<Mutex<Pdo>>,
    mut state_update_rx: mpsc::Receiver<Cia402Flags>,
    mut cmd_rx: broadcast::Receiver<MotorCommand>,
    new_setpoint_tx: mpsc::Sender<Setpoint>,
) {
    loop {
        tokio::select! {
            // Check for cia402 state update
            Some(new_state_flags) = state_update_rx.recv() => {
                trace!(
                    "Cia402 state update received, new cia402flags: {new_state_flags:?}",
                );

                if let Err(err) = pdo.lock().await.write_cia402_state_transition(&new_state_flags).await {
                    error!(
                        "Unable to write cia402 state transition: {err}",
                    );
                }
            }

            Ok(cmd) = cmd_rx.recv() => {
                trace!("update publisher received command: {cmd:?}");

                if let Err(err) = match cmd.clone() {
                    MotorCommand::Halt => {
                        let setpoint = PositionSetpoint {
                            flags: PositionFlagsCW::halt(),
                            target: 0,
                            profile_velocity: 0,
                        };
                        SetpointManager::write_new_setpoint(&new_setpoint_tx, Setpoint::ProfilePosition(setpoint)).await
                    }
                    MotorCommand::Home => {
                        let setpoint = HomingSetpoint {
                            flags: HomeFlagsCW::default(),
                        };
                        SetpointManager::write_new_setpoint(&new_setpoint_tx, Setpoint::Home(setpoint)).await
                    },
                    MotorCommand::MoveAbsolute { target, profile_velocity } => {
                        let setpoint = PositionSetpoint {
                            flags: PositionFlagsCW::absolute(),
                            target,
                            profile_velocity
                        };
                        SetpointManager::write_new_setpoint(&new_setpoint_tx,Setpoint::ProfilePosition(setpoint)).await
                    },
                    MotorCommand::MoveRelative { delta, profile_velocity } => {
                        let setpoint = PositionSetpoint {
                            flags: PositionFlagsCW::relative(),
                            target: delta,
                            profile_velocity
                        };
                        SetpointManager::write_new_setpoint(&new_setpoint_tx,Setpoint::ProfilePosition(setpoint)).await
                    },
                    MotorCommand::SetVelocity { target_velocity }=> {
                        let setpoint = VelocitySetpoint {
                            // flags: PositionModeFlags::relative(),
                            target_velocity,
                            // profile_velocity
                        };
                        SetpointManager::write_new_setpoint(&new_setpoint_tx, Setpoint::ProfileVelocity(setpoint)).await
                    },
                    MotorCommand::SetTorque { target_torque }=> {
                        let setpoint = TorqueSetpoint {
                            // flags: PositionModeFlags::relative(),
                            target_torque,
                            // profile_torque
                        };
                        SetpointManager::write_new_setpoint(&new_setpoint_tx, Setpoint::ProfileTorque(setpoint)).await
                    },
                    _ => {
                        trace!("update publisher ignoring command: {cmd:?}");
                        Ok(())
                    },
                } {
                    error!("Error handling command {cmd:?}: {err}");
                }
            }

            else => {
                error!("publish_updates: all channels closed, exiting task");
                break;
            }
        }
    }
}

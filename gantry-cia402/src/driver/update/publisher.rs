use tokio::sync::{
    broadcast,
    mpsc::{self},
    watch,
};
use tracing::*;

use crate::{
    comms::pdo::cmd::PdoCommand,
    driver::{
        command::MotorCommand,
        oms::{
            OperationMode,
            cyclic_pos::CyclicPositionSetpoint,
            cyclic_torque::CyclicTorqueSetpoint,
            cyclic_vel::CyclicVelocitySetpoint,
            home::{HomeFlagsCW, HomingSetpoint},
            position::{PositionFlagsCW, PositionSetpoint},
            setpoint::Setpoint,
            torque::TorqueSetpoint,
            velocity::VelocitySetpoint,
        },
        receiver::setpoint_manager::SetpointManager,
        state::{Cia402State, state_machine::Cia402Command},
    },
    error::DriveError,
};

/// Responsible for all CANopen communication to the drive
/// Receives updates from the Cia402 state machine and operational mode specific handler
/// It encodes these changes into the appropriate controlword bits or OD object
/// It then sends these changes out on the CANopen bus using the accessor
pub async fn publish_updates(
    pdo_tx: mpsc::Sender<PdoCommand>,
    mut state_update_rx: mpsc::Receiver<Cia402Command>,
    mut cmd_rx: broadcast::Receiver<MotorCommand>,
    new_setpoint_tx: mpsc::Sender<Setpoint>,
    cs_mode_tx: watch::Sender<OperationMode>,
    cia402_tx: mpsc::Sender<Cia402State>,
    node_id: u8,
) -> Result<(), DriveError> {
    loop {
        tokio::select! {
            // Check for cia402 state update
            Some(cia402_cmd) = state_update_rx.recv() => {
                match cia402_cmd {
                    // The cia402 SM detected a state update: Inform PDO system
                    Cia402Command::Update(flags) => {
                        trace!(
                            "Cia402 state update detected, new cia402flags: {flags:?}",
                        );

                        if let Err(err) = pdo_tx.send(PdoCommand::UpdateCia402Flags(flags)).await {
                             error!(
                                 "Unable to write cia402 state transition: {err}",
                             );
                        }
                    }

                    // The cia402 SM requested a state transition: pass on to PDO system
                    Cia402Command::Transition(flags) => {
                        trace!(
                            "Cia402 state transition requested, new cia402flags: {flags:?}",
                        );

                        if let Err(err) = pdo_tx.send(PdoCommand::WriteCia402Transition(flags)).await {
                             error!(
                                 "Unable to write cia402 state transition: {err}",
                             );
                        }
                    }
                };

            }

            Ok(cmd) = cmd_rx.recv() => {
                trace!("update publisher received command: {cmd:?}");

                if let Err(err) = match cmd.clone() {
                    MotorCommand::Enable => {
                        cia402_tx.send(Cia402State::OperationEnabled).await.map_err(|e| DriveError::Cia402SendError(e))
                    }
                    MotorCommand::Disable => {
                        cia402_tx.send(Cia402State::ReadyToSwitchOn).await.map_err(|e| DriveError::Cia402SendError(e))
                    }
                    MotorCommand::Cia402TransitionTo { target_state }  => {
                        cia402_tx.send(target_state).await.map_err(|e| DriveError::Cia402SendError(e))
                    }
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
                    MotorCommand::EnterCyclicSynchronousMode{ mode } => {
                        SetpointManager::enable_cyclic_synchronous_mode(
                                &cs_mode_tx,
                                mode,
                                node_id
                            ).await
                    },
                    MotorCommand::ExitCyclicSynchronousMode => {
                        SetpointManager::disable_cyclic_synchronous_mode(
                                &cs_mode_tx,
                                node_id
                            ).await
                    },
                    MotorCommand::CyclicSynchronousPosition { abs_target } => {
                        let setpoint = CyclicPositionSetpoint {
                                abs_target,
                        };
                        SetpointManager::write_new_setpoint(&new_setpoint_tx, Setpoint::CyclicPosition(setpoint)).await
                    },
                    MotorCommand::CyclicSynchronousVelocity { target } => {
                        let setpoint = CyclicVelocitySetpoint { target };
                        SetpointManager::write_new_setpoint(&new_setpoint_tx, Setpoint::CyclicVelocity(setpoint)).await
                    },
                    MotorCommand::CyclicSynchronousTorque { target } => {
                        let setpoint = CyclicTorqueSetpoint { target };
                        SetpointManager::write_new_setpoint(&new_setpoint_tx, Setpoint::CyclicTorque(setpoint)).await
                    },
                    MotorCommand::ResetFault => {
                        trace!("update publisher ignoring command: {cmd:?}");
                        Ok(())
                    },
                } {
                    error!("Error handling command {cmd:?}: {err}");
                }
            }

            else => {
                error!("publish_updates: all channels closed, exiting task");
                return Err(DriveError::InterTaskCommunicationError(String::from("publish_updates: all channels closed, exiting task")));
            }
        }
    }
}

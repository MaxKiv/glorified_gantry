use std::sync::Arc;

use oze_canopen::sdo_client::SdoClient;
use tokio::sync::{
    Mutex, broadcast,
    mpsc::{self},
    watch,
};
use tracing::*;

use crate::{
    comms::pdo::cmd::PdoCommand,
    driver::{
        command::MotorCommand,
        event::MotorEvent,
        nmt::NmtState,
        oms::{
            cyclic_pos::CyclicPositionSetpoint,
            cyclic_torque::CyclicTorqueSetpoint,
            cyclic_vel::CyclicVelocitySetpoint,
            home::{HomeFlagsCW, HomingSetpoint},
            position::{PositionFlagsCW, PositionSetpoint},
            setpoint::Setpoint,
            torque::TorqueSetpoint,
            velocity::VelocitySetpoint,
        },
        receiver::setpoint_manager::{SetpointManager, SetpointManagerModeTypes},
        state::Cia402Flags,
    },
    error::DriveError,
};

/// Responsible for all CANopen communication to the drive
/// Receives updates from the Cia402 state machine and operational mode specific handler
/// It encodes these changes into the appropriate controlword bits or OD object
/// It then sends these changes out on the CANopen bus using the accessor
pub async fn publish_updates(
    pdo_tx: mpsc::Sender<PdoCommand>,
    mut state_update_rx: mpsc::Receiver<Cia402Flags>,
    mut cmd_rx: broadcast::Receiver<MotorCommand>,
    new_setpoint_tx: mpsc::Sender<Setpoint>,
    cs_mode_tx: watch::Sender<SetpointManagerModeTypes>,
    nmt_tx: mpsc::Sender<NmtState>,
    event_rx: broadcast::Receiver<MotorEvent>,
    sdo: Arc<Mutex<SdoClient>>,
    node_id: u8,
) -> Result<(), DriveError> {
    loop {
        tokio::select! {
            // Check for cia402 state update
            Some(new_state_flags) = state_update_rx.recv() => {
                trace!(
                    "Cia402 state update received, new cia402flags: {new_state_flags:?}",
                );

               if let Err(err) = pdo_tx.send(PdoCommand::WriteCia402Transition(new_state_flags)).await {
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
                    MotorCommand::EnterCyclicSynchronousMode{ mode } => {
                        SetpointManager::enable_cyclic_synchronous_mode(
                                &cs_mode_tx,
                                mode,
                                &nmt_tx,
                                event_rx.resubscribe(),
                                sdo.clone(),
                                node_id
                            ).await
                    },
                    MotorCommand::ExitCyclicSynchronousMode => {
                        SetpointManager::disable_cyclic_synchronous_mode(
                                &cs_mode_tx,
                                &nmt_tx,
                                event_rx.resubscribe(),
                                sdo.clone(),
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
                return Err(DriveError::InterTaskCommunicationError(String::from("publish_updates: all channels closed, exiting task")));
            }
        }
    }
}

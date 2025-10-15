use tokio::sync::{
    broadcast,
    mpsc::{self},
};
use tracing::*;

use crate::{
    comms::pdo::Pdo,
    driver::{
        command::MotorCommand,
        oms::{
            HomeFlags, HomingSetpoint, PositionModeFlags, PositionSetpoint, TorqueSetpoint,
            VelocitySetpoint,
        },
        state::Cia402Flags,
    },
};

/// Responsible for all CANopen communication to the drive
/// Receives updates from the Cia402 state machine and operational mode specific handler
/// It encodes these changes into the appropriate controlword bits or OD object
/// It then sends these changes out on the CANopen bus using the accessor
pub async fn publish_updates(
    mut pdo: Pdo,
    mut state_update_rx: mpsc::Receiver<Cia402Flags>,
    mut cmd_rx: broadcast::Receiver<MotorCommand>,
) {
    loop {
        tokio::select! {
            // Check for cia402 state update
            Some(new_state_flags) = state_update_rx.recv() => {
                trace!(
                    "Cia402 state update received, new cia402flags: {new_state_flags:?}",
                );

                if let Err(err) = pdo.write_cia402_state_transition(new_state_flags).await {
                    error!(
                        "Unable to write cia402 state transition: {err}",
                    );
                }
            }

            Ok(cmd) = cmd_rx.recv() => {
                trace!("update publisher received command: {cmd:?}");

                if let Err(err) = match cmd {
                    MotorCommand::Halt => {
                        pdo.write_position_setpoint(PositionSetpoint {
                            flags: PositionModeFlags::halt(),
                            target: 0,
                            profile_velocity: 0,
                        }).await
                    }
                    MotorCommand::Home => {
                        pdo.write_homing_setpoint(HomingSetpoint {
                            flags: HomeFlags::default(),
                        }).await
                    },
                    MotorCommand::MoveAbsolute { target, profile_velocity } => {
                        pdo.write_position_setpoint(PositionSetpoint {
                            flags: PositionModeFlags::absolute(),
                            target,
                            profile_velocity
                        }).await
                    },
                    MotorCommand::MoveRelative { delta, profile_velocity } => {
                        pdo.write_position_setpoint(PositionSetpoint {
                            flags: PositionModeFlags::relative(),
                            target: delta,
                            profile_velocity
                        }).await
                    },
                    MotorCommand::SetVelocity { target_velocity }=> {
                        pdo.write_velocity_setpoint(VelocitySetpoint {
                            // flags: PositionModeFlags::relative(),
                            target_velocity,
                            // profile_velocity
                        }).await
                    },
                    MotorCommand::SetTorque { target_torque }=> {
                        pdo.write_torque_setpoint(TorqueSetpoint {
                            // flags: PositionModeFlags::relative(),
                            target_torque,
                            // profile_torque
                        }).await
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

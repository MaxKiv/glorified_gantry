use tokio::sync::mpsc::{self};
use tracing::*;

use crate::{
    comms::pdo::Pdo,
    driver::{oms::Setpoint, state::Cia402Flags},
};

/// Responsible for all CANopen communication to the drive
/// Receives updates from the Cia402 state machine and operational mode specific handler
/// It encodes these changes into the appropriate controlword bits or OD object
/// It then sends these changes out on the CANopen bus using the accessor
pub async fn publish_updates(
    mut pdo: Pdo,
    mut state_update_rx: mpsc::Receiver<Cia402Flags>,
    mut setpoint_update_rx: mpsc::Receiver<Setpoint>,
) {
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
        // Check for cia402 state transition requests from the user
        Some(new_setpoint) = setpoint_update_rx.recv() => {
            trace!(
                "Setpoint update received, new setpoint {new_setpoint:?}",
            );

            if let Err(err) = match new_setpoint {
                Setpoint::ProfilePosition(position_setpoint) => {
                    pdo.write_position_setpoint(position_setpoint).await
                }
                Setpoint::ProfileVelocity(velocity_setpoint) => {
                    pdo.write_velocity_setpoint(velocity_setpoint).await
                }
                Setpoint::ProfileTorque(torque_setpoint) => {
                    pdo.write_torque_setpoint(torque_setpoint).await
                }
            } {
                error!(
                    "Unable to write setpoint update: {err}",
                );
            }

        }
    }
}

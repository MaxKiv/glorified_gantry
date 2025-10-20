use std::time::Duration;

use oze_canopen::{
    interface::CanOpenInterface,
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
};
use tokio::{
    sync::{broadcast, mpsc},
    time::timeout,
};
use tracing::*;

use crate::{driver::event::MotorEvent, error::DriveError};

#[derive(Debug, PartialEq, Clone)]
pub enum NmtState {
    Bootup,
    Stopped,
    PreOperational,
    Operational,
}

impl Into<NmtCommandSpecifier> for NmtState {
    fn into(self) -> NmtCommandSpecifier {
        match self {
            NmtState::Bootup => NmtCommandSpecifier::ResetCommunication,
            NmtState::Stopped => NmtCommandSpecifier::StopRemoteNode,
            NmtState::PreOperational => NmtCommandSpecifier::EnterPreOperational,
            NmtState::Operational => NmtCommandSpecifier::StartRemoteNode,
        }
    }
}

pub async fn nmt_task(
    node_id: u8,
    canopen: CanOpenInterface,
    mut nmt_rx: mpsc::Receiver<NmtState>,
    mut event_rx: broadcast::Receiver<MotorEvent>,
) -> Result<(), DriveError> {
    let mut current_state = NmtState::PreOperational;
    loop {
        tokio::select! {
            // Process NMT state updates from feedback task
            event = event_rx.recv() => {
                if let Ok(event) = event {
                    match event {
                        MotorEvent::NmtStateUpdate(nmt_state) => {
                            trace!("NMT: Received NMT state update: {nmt_state:?}");

                            let new_state = nmt_state;
                            trace!(
                                "NMT state update received, old -> new state: {:?} -> {new_state:?}",
                                current_state
                            );
                            current_state = new_state;
                        },

                        _ => continue,
                    }
                }
            }

            // Set device to requested state
            state = nmt_rx.recv() => {
                trace!("Received NMT state request: {state:?}");
                if let Some(state) = state {
                    match canopen.send_nmt(
                        NmtCommand::new(state.clone().into(), node_id)
                    ).await {
                        Ok(_) => {
                            trace!("Send NMT state request: {state:?} to node {node_id}");
                        }
                        Err(err) => {
                            trace!("Error sending NMT state request to node {node_id}: {err:?}");
                        }
                    }
                }

            }

        }
    }
}

pub async fn set_to_nmt_state(
    state: NmtState,
    nmt_tx: &mpsc::Sender<NmtState>,
    mut event_rx: broadcast::Receiver<MotorEvent>,
) -> Result<(), DriveError> {
    const NMT_SWITCH_TIMEOUT: Duration = Duration::from_secs(1);
    const NMT_SWITCH_ATTEMPTS: usize = 10;

    let mut attempt = 0;

    loop {
        // Notify the NMT task of the required NMT state, this handle the switching
        nmt_tx
            .send(state.clone())
            .await
            .map_err(|err| DriveError::NMTSendError(state.clone(), err))?;

        // Wait for event indicating correct NMT state
        match timeout(NMT_SWITCH_TIMEOUT, event_rx.recv()).await {
            Ok(Ok(MotorEvent::NmtStateUpdate(new_state))) => {
                trace!("new_state: {new_state:?}");
                // Got an event within the timeout
                if new_state == state {
                    return Ok(());
                }
            }
            Ok(Ok(_)) => {
                // Non-NMT event
            }
            Ok(Err(err)) => {
                // The channel closed before we got an event
                error!("Startup NMT PRE-OP: {err}");
                return Err(DriveError::NMTSwitchError(state));
            }
            Err(_) => {
                // Timeout expired, try again
                warn!("Startup NMT PRE-OP: Timed out waiting for event");
            }
        }

        attempt += 1;
        if attempt >= NMT_SWITCH_ATTEMPTS {
            error!(
                "Failed to switch device into NMT {state:?} after {NMT_SWITCH_ATTEMPTS} attempts, aborting"
            );
            return Err(DriveError::NMTSwitchError(state));
        }
    }
}

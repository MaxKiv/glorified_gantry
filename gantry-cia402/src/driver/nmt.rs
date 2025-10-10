use std::time::Duration;

use oze_canopen::{
    interface::CanOpenInterface,
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
};
use tokio::{
    sync::{broadcast, mpsc},
    task,
};
use tracing::*;

use crate::{
    driver::{event::MotorEvent, receiver::StatusWord},
    error::DriveError,
};

#[derive(Debug, PartialEq, Clone)]
pub enum NmtState {
    Bootup,
    Stopped,
    PreOperational,
    Operational,
}

impl From<StatusWord> for NmtState {
    fn from(status: StatusWord) -> Self {
        if status.intersects(
            StatusWord::OPERATION_ENABLED
                | StatusWord::SWITCHED_ON
                | StatusWord::READY_TO_SWITCH_ON,
        ) {
            NmtState::Operational
        } else if status.intersects(StatusWord::SWITCHED_ON | StatusWord::READY_TO_SWITCH_ON) {
            NmtState::PreOperational
        } else {
            NmtState::Stopped
        }
    }
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
) {
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

/// Makes sure the device is set to NmtState::OP
pub async fn transition_to_operational(
    node_id: u8,
    canopen: CanOpenInterface,
    current_state: &NmtState,
) -> Result<(), DriveError> {
    if *current_state != NmtState::Operational {
        trace!(
            "Motor with node id {} is in NMT state: {:?} - Requesting NmtState::Operational",
            node_id, current_state,
        );
        match canopen
            .send_nmt(NmtCommand::new(
                NmtCommandSpecifier::StartRemoteNode,
                node_id,
            ))
            .await
        {
            Ok(_) => {
                trace!("Send NMT Operational to motor with node id {}", node_id);
            }
            Err(err) => {
                error!(
                    "Error setting motor {} to NMT Operational: {err:?}",
                    node_id
                );

                return Err(DriveError::CanOpen(err));
            }
        }
    }

    Ok(())
}

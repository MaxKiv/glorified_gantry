use std::time::Duration;

use oze_canopen::{
    interface::CanOpenInterface,
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
};
use tokio::{
    sync::{broadcast, mpsc},
    task, time,
};
use tracing::*;

use crate::{
    driver::{event::MotorEvent, feedback::StatusWord},
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

pub struct HeartBeat {
    pub data: u8,
}

impl From<HeartBeat> for NmtState {
    fn from(heartbeat: HeartBeat) -> Self {
        match heartbeat.data {
            0x04 => NmtState::Stopped,
            0x05 => NmtState::Operational,
            0x7f => NmtState::PreOperational,
            0x00 => NmtState::Stopped,
            _ => NmtState::Stopped,
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

pub struct Nmt;

impl Nmt {
    /// Start the NMT task responsible for putting the device into NmtState::Operational, which is a
    /// prerequisite for doing anything with a cia402 motordriver
    pub fn start(
        node_id: u8,
        canopen: CanOpenInterface,
        nmt_rx: mpsc::Receiver<NmtState>,
        event_rx: broadcast::Receiver<MotorEvent>,
    ) -> task::JoinHandle<()> {
        trace!("Starting NMT State Machine task for motor with node id {node_id}");

        task::spawn(Nmt::run(node_id, canopen, nmt_rx, event_rx))
    }

    pub async fn run(
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

                            MotorEvent::StatusWord(status_word) => {
                                trace!("NMT: Received statusword update: {status_word:?}");
                                let new_state: NmtState = status_word.into();
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

            tokio::time::sleep(Duration::from_millis(250)).await;
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
}

use std::time::Duration;

use oze_canopen::{
    interface::CanOpenInterface,
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
};
use tokio::{
    sync::{broadcast, mpsc::Receiver},
    task,
};
use tracing::*;

use crate::{
    driver::{event::MotorEvent, feedback::StatusWord},
    error::DriveError,
};

#[derive(Debug, PartialEq, Clone)]
pub enum NmtState {
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

pub struct Nmt;

impl Nmt {
    /// Start the NMT task responsible for putting the device into NmtState::Operational, which is a
    /// prerequisite for doing anything with a cia402 motordriver
    pub fn start(
        node_id: u8,
        canopen: CanOpenInterface,
        event_rx: broadcast::Receiver<MotorEvent>,
    ) -> task::JoinHandle<()> {
        trace!("Starting NMT State Machine task for motor with node id {node_id}");

        task::spawn(Nmt::run(node_id, canopen, event_rx))
    }

    pub async fn run(
        node_id: u8,
        canopen: CanOpenInterface,
        mut event_rx: broadcast::Receiver<MotorEvent>,
    ) {
        let mut current_state = NmtState::PreOperational;

        loop {
            tokio::select! {
                // Process NMT state updates from feedback task
                event = event_rx.recv() => {
                    if let Ok(MotorEvent::StatusWord(statusword)) = event {
                        let new_state: NmtState = statusword.into();
                        trace!(
                            "NMT state update received, old -> new state: {:?} -> {new_state:?}",
                            current_state
                        );
                        current_state = new_state;
                    }
                }
                // Continously attempt to put the motor in NmtState::Operational
                Err(err) = Nmt::transition_to_operational(node_id, canopen.clone(), &current_state)=> {
                    error!(
                        "Error transitioning device with node id {} to NMT::Operational: {err}",
                        node_id
                    );
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

use oze_canopen::{
    canopen,
    interface::CanOpenInterface,
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
};
use tokio::{sync::mpsc::Receiver, task};

use crate::error::DriveError;

#[derive(Debug)]
pub enum NmtState {
    Stopped,
    PreOperational,
    Operational,
}

pub struct Nmt {
    node_id: u8,
    canopen: CanOpenInterface,
    current_state: NmtState,
    nmt_rx: Receiver<NmtState>,
}

impl Nmt {
    pub fn init(
        node_id: u8,
        canopen: CanOpenInterface,
        nmt_rx: Receiver<NmtState>,
    ) -> task::JoinHandle<Result<(), DriveError>> {
        let mut nmt = Self {
            node_id,
            canopen,
            current_state: NmtState::PreOperational,
            nmt_rx,
        };

        let nmt_handler = task::spawn(nmt.run());

        nmt_handler
    }

    pub async fn run(&mut self) -> Result<(), DriveError> {
        loop {
            // Process NMT state updates from handle_feedback
            if let Some(new_state) = self.nmt_rx.recv().await {
                trace!(
                    "NMT state update received, old -> new state: {self.current_state:?} -> {new_state:?}",
                );
                self.current_state = new_state;
            }

            // Continously attempt to put the motor in NmtState::Operational
            let _ = self.transition_to_operational().await;
        }
    }

    /// Makes sure the device is set to NmtState::OP
    pub async fn transition_to_operational(&mut self) -> Result<(), DriveError> {
        if self.current_state != NmtState::Operational {
            trace!(
                "Motor with node id {} is in NMT state: {:?} - Requesting NmtState::Operational",
                self.node_id, self.current_state,
            );
            match self
                .canopen
                .send_nmt(NmtCommand::new(
                    NmtCommandSpecifier::StartRemoteNode,
                    self.node_id,
                ))
                .await
            {
                Ok(_) => {
                    trace!(
                        "Send NMT Operational to motor with node id {}",
                        self.node_id
                    );
                }
                Err(err) => {
                    error!(
                        "Error setting motor {} to NMT Operational: {err}",
                        self.node_id
                    );

                    return DriveError::CanOpen(err);
                }
            }
        }

        Ok(())
    }
}

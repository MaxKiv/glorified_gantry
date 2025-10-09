use std::time::Duration;

use gantry_cia402::{
    comms::pdo::mapping::PdoMapping,
    driver::{event::MotorEvent, receiver::subscriber::handle_feedback},
};
use oze_canopen::interface::CanOpenInterface;
use thiserror::Error;
use tokio::{
    sync::broadcast::{self, error::RecvError},
    task::{self, JoinHandle},
    time::{self, Instant, error::Elapsed},
};
use tracing::*;

#[derive(Debug, Error)]
pub enum TestError {
    #[error("Timeout waiting for event: {0:?}: {1:?}")]
    Timeout(MotorEvent, Option<Elapsed>),
    #[error("Broadcast lag waiting for event: {0:?}: {1:?}")]
    BroadcastLagged(MotorEvent, RecvError),
    #[error("Broadcast closed waiting for event: {0:?}: {1:?}")]
    BroadcastClosed(MotorEvent, RecvError),
}

pub async fn wait_for_event(
    mut event_rx: broadcast::Receiver<MotorEvent>,
    watch_for: MotorEvent,
    timeout: Duration,
) -> Result<(), TestError> {
    let deadline = Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            warn!("Timeout when waiting for event: {watch_for:?}");
            return Err(TestError::Timeout(watch_for, None));
        }

        let recv_future = event_rx.recv();
        let result = time::timeout(remaining, recv_future).await;

        match result {
            Ok(Ok(event)) => {
                if event == watch_for {
                    return Ok(());
                }
                // else keep looping for the next one
            }
            Ok(Err(err @ broadcast::error::RecvError::Lagged(_))) => {
                // Messages were missed, continue to next one
                error!("Lagged in wait_for_event, indicates serious issue");
                return Err(TestError::BroadcastLagged(watch_for, err));
            }
            Ok(Err(err @ broadcast::error::RecvError::Closed)) => {
                error!("Event channel closed in wait_for_event");
                return Err(TestError::BroadcastClosed(watch_for, err));
            }
            Err(err) => {
                warn!("Timeout when waiting for event: {watch_for:?}");
                return Err(TestError::Timeout(watch_for, Some(err)));
            }
        }
    }
}

/// Start the device feedback task responsible for receiving and parsing device feedback and broadcasting these as events
pub fn start_feedback_task(
    canopen: CanOpenInterface,
    node_id: u8,
    tpdo_mapping_set: &'static [PdoMapping],
) -> (JoinHandle<()>, broadcast::Receiver<MotorEvent>) {
    // Initialize output interfaces
    let (event_tx, event_rx): (
        broadcast::Sender<MotorEvent>,
        broadcast::Receiver<MotorEvent>,
    ) = tokio::sync::broadcast::channel(10);

    trace!("Starting device feedback handler for motor with node id {node_id}");
    (
        task::spawn(handle_feedback(
            node_id,
            canopen,
            tpdo_mapping_set,
            event_tx,
        )),
        event_rx,
    )
}

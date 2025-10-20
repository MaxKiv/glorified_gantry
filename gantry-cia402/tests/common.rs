use std::time::Duration;

use gantry_cia402::{
    comms::{
        pdo::mapping::{PDOSet, PdoMapping, minimal::MINIMAL_CYCLIC_SYNCHRONOUS_PDO_SET},
        sdo::SdoAction,
    },
    driver::{
        event::MotorEvent, nmt::NmtState, receiver::subscriber::handle_feedback, spawn_logged,
        startup,
    },
    error::DriveError,
};
use oze_canopen::{error::CoError, interface::CanOpenInterface};
use thiserror::Error;
use tokio::{
    sync::{
        broadcast::{self, error::RecvError},
        mpsc::error::SendError,
    },
    task::{self, JoinHandle},
    time::{self, Instant, error::Elapsed},
};
use tracing::*;

// Default test parameters
pub const CAN_INTERFACE: &str = "can0";
pub const CAN_BITRATE: u32 = 1_000_000;
pub const NODE_ID: u8 = 3;
pub const PARAMS: &[SdoAction] = startup::params::PARAMS;
pub const TIMEOUT: Duration = Duration::from_secs(5);
pub const CYCLIC_PDOS: PDOSet = MINIMAL_CYCLIC_SYNCHRONOUS_PDO_SET;
pub const SYNC_PERIOD: Duration = Duration::from_millis(1000);

#[derive(Debug, Error)]
pub enum TestError {
    #[error("Error from CANOpen: {0:?}")]
    CANOpenError(CoError),
    #[error("Error from CANOpen: {0:?}")]
    ConversionError(String),
    #[error("Timeout waiting for event: {0:?}: {1:?}")]
    Timeout(MotorEvent, Option<Elapsed>),
    #[error("Broadcast lag waiting for event: {0:?}: {1:?}")]
    BroadcastLagged(MotorEvent, RecvError),
    #[error("Broadcast closed waiting for event: {0:?}: {1:?}")]
    BroadcastClosed(MotorEvent, RecvError),
    #[error("Error switching to NMT state: {0:?}: {1:?}")]
    NMTSendError(NmtState, SendError<NmtState>),
    #[error("Generic test error")]
    Generic,
}

/// Start the device feedbac task responsible for receiving and parsing device feedback and broadcasting these as events
pub fn start_feedback_task(
    canopen: CanOpenInterface,
    node_id: u8,
    tpdo_mapping_set: &'static [PdoMapping],
) -> (
    JoinHandle<Result<(), DriveError>>,
    broadcast::Receiver<MotorEvent>,
) {
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

pub async fn sync_loop(
    sync_tx: broadcast::Sender<Instant>,
    canopen: CanOpenInterface,
) -> Result<(), DriveError> {
    let mut interval = time::interval(SYNC_PERIOD);
    loop {
        interval.tick().await;

        // 1️⃣ broadcast to all drivers
        sync_tx
            .send(Instant::now())
            .map_err(|_| DriveError::ViolatedInvariant(String::from("unable to send SYNC")))?;

        // 2️⃣ send SYNC frame on bus
        canopen
            .send_sync()
            .await
            .map_err(|_| DriveError::ViolatedInvariant(String::from("unable to send SYNC")))?;
    }
}

pub fn start_sync_master(canopen: CanOpenInterface) -> broadcast::Receiver<Instant> {
    let (sync_tx, sync_rx) = tokio::sync::broadcast::channel(10);

    spawn_logged("SYNC", async move { sync_loop(sync_tx, canopen).await });

    sync_rx
}

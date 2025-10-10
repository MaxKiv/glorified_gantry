use std::time::Duration;

use gantry_cia402::{
    comms::{
        pdo::mapping::{
            PdoMapping,
            custom::{CUSTOM_RPDOS, CUSTOM_TPDOS},
        },
        sdo::SdoAction,
    },
    driver::{event::MotorEvent, nmt::NmtState, receiver::subscriber::handle_feedback, startup},
    log::{log_canopen_pretty, log_events},
};
use oze_canopen::{error::CoError, interface::CanOpenInterface};
use thiserror::Error;
use tokio::{
    sync::{
        broadcast::{self, error::RecvError},
        mpsc::{self, error::SendError},
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
pub const TPDOS: &[PdoMapping; 4] = CUSTOM_TPDOS;
pub const RPDOS: &[PdoMapping; 4] = CUSTOM_RPDOS;

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

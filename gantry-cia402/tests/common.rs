use std::time::Duration;

use gantry_cia402::{
    comms::{
        pdo::mapping::{PdoMapping, PdoSet, minimal::MINIMAL_CYCLIC_SYNCHRONOUS_PDO_SET},
        sdo::SdoAction,
    },
    driver::{
        event::MotorEvent, identifier::Cia402Identifier, nmt::NmtState,
        receiver::subscriber::handle_feedback, startup,
    },
    error::DriveError,
};
use gantry_demo::config::{
    TEST_SETUP_DEVICE_NAME, TEST_SETUP_MOTOR_TYPE, TEST_SETUP_PROFILE_NUMBER,
};
use oze_canopen::{error::CoError, interface::CanOpenInterface};
use thiserror::Error;
use tokio::{
    sync::{
        broadcast::{self, error::RecvError},
        mpsc::error::SendError,
    },
    task::{self, JoinHandle},
    time::error::Elapsed,
};
use tracing::*;

// Default test parameters
pub const CAN_INTERFACE: &str = "can0";
pub const CAN_BITRATE: u32 = 1_000_000;
pub const TEST_MOTOR: Cia402Identifier = Cia402Identifier {
    node_id: 1,
    device_profile_number: TEST_SETUP_PROFILE_NUMBER,
    motor_type: TEST_SETUP_MOTOR_TYPE,
    device_name: TEST_SETUP_DEVICE_NAME,
};
pub const PARAMS: &[SdoAction] = startup::params::TEST_PARAMS;
pub const COMMS_TIMEOUT: Duration = Duration::from_secs(5);
pub const POS_TIMEOUT: Duration = Duration::from_secs(30);
pub const HOMING_TIMEOUT: Duration = Duration::from_secs(60);
pub const CYCLIC_PDOS: PdoSet = MINIMAL_CYCLIC_SYNCHRONOUS_PDO_SET;

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

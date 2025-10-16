pub mod frame;
pub mod log;
pub mod pdo;
pub mod pdo_message;
pub mod sdo_response;

use oze_canopen::canopen::{NodeId, RxMessage};
use tokio::time::Instant;

use crate::{
    driver::{
        nmt::NmtState,
        receiver::parse::{
            pdo_message::ParsedPDO,
            sdo_response::{SdoRequest, SdoResponse},
        },
    },
    od::entry::ODEntry,
};

#[derive(Debug)]
pub struct ParseError(anyhow::Error);

pub struct Frame {
    pub timestamp: Instant,
    pub node_id: Option<NodeId>, // Node id this message is for, None means broadcast
    pub message: MessageType,
}

#[derive(Debug)]
pub enum MessageType {
    NmtControl(NmtControlMessage),
    Sync(SyncMessage), // No node id
    EMCY(EmergencyMessage),
    TSDO(SdoResponse),
    RSDO(SdoRequest),
    PDO(ParsedPDO),
    NmtMonitor(NmtMonitorMessage),
    Unknown(RxMessage), // No node id
}
#[derive(Debug)]

pub struct NmtControlMessage {
    pub requested_state: NmtState,
}

#[derive(Debug)]
pub struct NmtMonitorMessage {
    pub current_state: NmtState,
}

#[derive(Debug)]
pub struct SyncMessage;

#[derive(Debug)]
pub struct EmergencyMessage {
    pub error: EMCY,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EMCY {
    Undervoltage,
    InterlockError,
    SoftwareReset,
    InternalSoftwareError,
    RatedCurrentNotSet,
    BallastResistorOverload,
    MotorBlocked,
    InternalCorrectionFactorMissing,
    Sensor1Fault,
    Sensor2Fault,
    SensorNFault,
    NonvolatileMemoryFull,
    FieldbusError,
    HeartbeatError,
    SlaveTimeout,
    PdoLengthError,
    PdoLengthExceeded,
    UnexpectedSyncLength,
    SpeedMonitoringError,
    FollowingErrorTooLarge,
    LimitSwitchExceeded,
    NoFurtherPendingErrors,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TPDOMessage {
    pub num: usize,
    pub data: [u8; 8],
    pub dlc: usize,
}

#[derive(Debug, Clone)]
pub struct RPDOMessage {
    num: usize,
    data: [u8; 8],
    dlc: usize,
}

#[derive(Debug)]
pub struct SdoMessage {
    data: [u8; 8],
    dlc: usize,
    value: Option<ODEntry>,
}

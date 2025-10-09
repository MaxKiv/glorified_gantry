pub mod log;
pub mod parse;
pub mod sdo_response;

use oze_canopen::canopen::{NodeId, RxMessage};
use tokio::time::Instant;

use crate::{
    comms::pdo::mapping::PdoType,
    driver::{
        nmt::NmtState,
        receiver::frame::sdo_response::{SdoRequest, SdoResponse},
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
    TPDO(TPDOMessage),
    RPDO(RPDOMessage),
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

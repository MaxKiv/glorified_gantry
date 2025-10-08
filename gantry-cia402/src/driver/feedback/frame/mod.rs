pub mod log;
pub mod parse;

use oze_canopen::canopen::{NodeId, RxMessage};
use tokio::time::Instant;

use crate::{comms::pdo::mapping::PdoType, driver::nmt::NmtState};

pub enum Frame {
    NmtControl(NmtControlMessage),
    Sync(SyncMessage),
    EMCY(EmergencyMessage),
    TSDO(SdoMessage),
    RSDO(SdoMessage),
    TPDO(TPDOMessage),
    RPDO(RPDOMessage),
    NmtMonitor(NmtMonitorMessage),
    Unknown(RxMessage),
}

pub struct NmtControlMessage {
    timestamp: Instant,
    requested_state: NmtState,
    to: NodeId,
}

pub struct NmtMonitorMessage {
    timestamp: Instant,
    from: NodeId,
    current_state: NmtState,
}

pub struct SyncMessage {
    timestamp: Instant,
}

#[derive(Debug)]
pub struct EmergencyMessage {
    timestamp: Instant,
    from: NodeId,
    error: EMCY,
}

#[derive(Debug)]
pub enum EMCY {
    Undervoltage,
    Unknown,
}

pub struct TPDOMessage {
    timestamp: Instant,
    from: NodeId,
    num: usize,
    data: [u8; 8],
    dlc: usize,
}

pub struct RPDOMessage {
    timestamp: Instant,
    from: NodeId,
    num: usize,
    data: [u8; 8],
    dlc: usize,
}

pub struct SdoMessage {
    timestamp: Instant,
    from: NodeId,
    data: [u8; 8],
    dlc: usize,
}

pub mod frame;
pub mod od;
pub mod pdo;
pub mod sdo;

use socketcan::CanDataFrame;

use crate::canopen::{
    frame::NodeId,
    pdo::message::RawPdoMessage,
    sdo::{SdoRequest, SdoResponse},
};

#[derive(Debug)]
pub enum MessageType {
    NmtControl(NmtControlMessage),
    Sync(SyncMessage), // No node id
    EMCY(EmergencyMessage),
    TSDO(SdoResponse),
    RSDO(SdoRequest),
    PDO(RawPdoMessage),
    NmtMonitor(NmtMonitorMessage),
    Unknown(CanDataFrame), // No node id
}

#[derive(Debug)]
pub struct NmtControlMessage {
    pub node_id: NodeId,
    pub requested_state: NmtState,
}

#[derive(Debug)]
pub struct NmtMonitorMessage {
    pub node_id: NodeId,
    pub current_state: NmtState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NmtState {
    Bootup,
    Stopped,
    PreOperational,
    Operational,
}
impl NmtState {
    fn from_nmt_command_frame(frame_data: &[u8]) -> Self {
        match frame_data[0] {
            0x01 => NmtState::Operational,
            0x02 => NmtState::Stopped,
            0x80 => NmtState::PreOperational,
            _ => NmtState::PreOperational,
        }
    }

    fn from_node_monitoring_frame(frame_data: &[u8]) -> Self {
        match frame_data[0] {
            0x00 => NmtState::Bootup,
            0x04 => NmtState::Stopped,
            0x05 => NmtState::Operational,
            0x7F => NmtState::PreOperational,
            _ => NmtState::PreOperational,
        }
    }
}

#[derive(Debug)]
pub struct SyncMessage;

#[derive(Debug)]
pub struct EmergencyMessage {
    pub node_id: NodeId,
    pub error: EMCY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl EMCY {
    fn from_error_code(error_code: u16) -> EMCY {
        match error_code {
            0x0 => EMCY::NoFurtherPendingErrors,
            0x3100 => EMCY::Undervoltage,
            0x8210 => EMCY::PdoLengthError,
            0x8220 => EMCY::PdoLengthExceeded,
            0x5440 => EMCY::InterlockError,
            0x6010 => EMCY::SoftwareReset,
            0x6100 => EMCY::InternalSoftwareError,
            0x6320 => EMCY::RatedCurrentNotSet,
            0x7113 => EMCY::BallastResistorOverload,
            0x7121 => EMCY::MotorBlocked,
            0x7200 => EMCY::InternalCorrectionFactorMissing,
            0x7305 => EMCY::Sensor1Fault,
            0x7306 => EMCY::Sensor2Fault,
            0x7307 => EMCY::SensorNFault,
            0x7600 => EMCY::NonvolatileMemoryFull,
            0x8100 => EMCY::FieldbusError,
            0x8130 => EMCY::HeartbeatError,
            0x8200 => EMCY::SlaveTimeout,
            0x8240 => EMCY::UnexpectedSyncLength,
            0x8400 => EMCY::SpeedMonitoringError,
            0x8611 => EMCY::FollowingErrorTooLarge,
            0x8612 => EMCY::LimitSwitchExceeded,
            _ => EMCY::Unknown,
        }
    }
}

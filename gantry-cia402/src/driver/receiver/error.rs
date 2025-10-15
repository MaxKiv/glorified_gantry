use thiserror::Error;

use crate::driver::receiver::parse::{RPDOMessage, TPDOMessage, pdo_message::ParsedPDO};

#[derive(Debug, Error)]
pub enum ReceiverError {
    // #[error("Timeout waiting for event: {0:?}: {1:?}")]
    // Timeout(MotorEvent, Option<Elapsed>),
    #[error("Received unknown / unmapped TPDO: {0:?}")]
    UnknownTPDO(TPDOMessage),
    #[error("Received unknown / unmapped RPDO: {0:?}")]
    UnknownRPDO(RPDOMessage),
    #[error("Received unknown / unmapped PDO: {0:?}")]
    UnknownPDO(ParsedPDO),
}

use oze_canopen::{error::CoError, transmitter::TxPacket};
use thiserror::Error;
use tokio::sync::mpsc::error::SendTimeoutError;

use crate::driver::state::Cia402State;

#[derive(Debug, Error)]
pub enum DriveError {
    #[error("Invalid state transition from {0:?} to {1:?}")]
    InvalidTransition(Cia402State, Cia402State),
    #[error("Invalid Operation Mode Specific Operation: {0}")]
    OperationModeSpecific(String),
    #[error("CANopen communication error: {0:?}")]
    CanOpen(CoError),
    #[error("Timeout Sending CANopen packet {0:?}")]
    Timeout(SendTimeoutError<TxPacket>),
    #[error("Invalid conversion of {0:?} into integer")]
    Conversion(Vec<u8>),
    #[error("Invariant violated: {0}")]
    ViolatedInvariant(String),
}

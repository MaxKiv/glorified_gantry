use oze_canopen::{error::CoError, transmitter::TxPacket};
use thiserror::Error;
use tokio::{
    sync::{
        broadcast::error::RecvError,
        mpsc::error::{SendError, SendTimeoutError},
    },
    time::error::Elapsed,
};

use crate::driver::{event::MotorEvent, nmt::NmtState, state::Cia402State};

#[derive(Debug, Error)]
pub enum DriveError {
    #[error("Invalid state transition from {0:?} to {1:?}")]
    InvalidTransition(Cia402State, Cia402State),
    #[error("Invalid Operation Mode Specific Operation: {0}")]
    OperationModeSpecific(String),
    #[error("CANopen communication error: {0:?}")]
    CanOpen(CoError),
    #[error("Timeout Sending CANopen packet {0:?}")]
    CanOpenTimeout(SendTimeoutError<TxPacket>),
    #[error("Invalid conversion of {0:?} into integer")]
    Conversion(Vec<u8>),
    #[error("Invariant violated: {0}")]
    ViolatedInvariant(String),
    #[error("Error from CANOpen: {0:?}")]
    CANOpenError(CoError),
    #[error("Error from CANOpen: {0:?}")]
    ConversionError(String),
    #[error("Timeout waiting for event: {0:?}: {1:?}")]
    EventTimeout(MotorEvent, Option<Elapsed>),
    #[error("Broadcast lag waiting for event: {0:?}: {1:?}")]
    BroadcastLagged(MotorEvent, RecvError),
    #[error("Broadcast closed waiting for event: {0:?}: {1:?}")]
    BroadcastClosed(MotorEvent, RecvError),
    #[error("Error switching to NMT state: {0:?}: {1:?}")]
    NMTSendError(NmtState, SendError<NmtState>),
}

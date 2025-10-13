use oze_canopen::{error::CoError, transmitter::TxPacket};
use thiserror::Error;
use tokio::{
    sync::{
        broadcast::{self, error::RecvError},
        mpsc::{
            self,
            error::{SendError, SendTimeoutError},
        },
    },
    time::error::Elapsed,
};

use crate::driver::{
    command::MotorCommand, event::MotorEvent, nmt::NmtState, receiver::StatusWord,
    state::Cia402State,
};

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
    #[error("Unable to decode {0:?} into Cia402State")]
    Cia402StateDecode(StatusWord),
    #[error("Unable to send motor command {0:?}")]
    CommandError(broadcast::error::SendError<MotorCommand>),
    #[error("Unable to send Cia402 State to Cia402 SM {0:?}")]
    Cia402SendError(mpsc::error::SendError<Cia402State>),
    #[error("No viable transition path from {0:?} to {1:?}")]
    Cia402TransitionError(Cia402State, Cia402State),
    #[error("Timeout asking cia402 SM to transition from {0:?} to {1:?}")]
    Cia402TransitionTimeout(Cia402State, Cia402State),
}

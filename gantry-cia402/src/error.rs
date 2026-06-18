use oze_canopen::{error::CoError, transmitter::TxPacket};
use thiserror::Error;
use tokio::{
    sync::{
        broadcast::{self, error::RecvError},
        mpsc::{
            self,
            error::{SendError, SendTimeoutError},
        },
        watch,
    },
    time::error::Elapsed,
};

use crate::{
    comms::{pdo::mapping::PdoMapping, sdo::SdoAction},
    driver::{
        command::MotorCommand,
        event::MotorEvent,
        identifier::{
            Cia402Identifier, CiaProfileNumber, InvalidCiaProfileNumber, InvalidMotorTypeError,
            MotorType,
        },
        nmt::NmtState,
        oms::{OperationMode, setpoint::Setpoint},
        receiver::{StatusWord, setpoint_manager::SetpointManagerModeTypes},
        state::Cia402State,
    },
    od::entry::ODEntry,
};

#[derive(Debug, Error)]
pub enum InitialisationError {
    #[error("PDO Manager Initialisation Error: Missing required PDO mapping: {0:?}")]
    MissingRequiredPDOMapping(ODEntry),
    #[error("Unable to construct an SDO client for node id {0}")]
    SdoClientConstructionFailed(Cia402Identifier),
    #[error("Unable to put drive (node {0}) into NMT PreOperational, required for parametrisation")]
    ParametrisationNMTPreOp(Cia402Identifier),
    #[error("Unable to put drive (node {0}) into NMT Operational, required after parametrisation")]
    ParametrisationNMTOp(Cia402Identifier),
    #[error("Wrong motor type: ({0:?}) for motor {1}")]
    ParametrisationWrongMotorType(MotorType, Cia402Identifier),
    #[error("Invalid motor type: ({0:?}) for motor {1}")]
    ParametrisationInvalidMotorType(InvalidMotorTypeError, Cia402Identifier),
    #[error("Wrong cia profile number: ({0:?}) for motor {1}")]
    ParametrisationWrongCiaProfileNumber(CiaProfileNumber, Cia402Identifier),
    #[error("Invalid cia profile number: ({0:?}) for motor {1}")]
    ParametrisationInvalidCiaProfileNumber(InvalidCiaProfileNumber, Cia402Identifier),
    #[error("Wrong device name: ({0}) for motor {1}")]
    ParametrisationWrongDeviceName(String, Cia402Identifier),
    #[error("Unable to communicate with motor {0}")]
    ParametrisationCommunicationFailure(Cia402Identifier),
    #[error("Unable to parametrise motor {0}")]
    ParametrisationError(Cia402Identifier),
}

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
    #[error("Timeout waiting for event to match predicate")]
    EventMatchesTimeout,
    #[error("Broadcast lag waiting for event: {0:?}: {1:?}")]
    BroadcastLagged(Option<MotorEvent>, RecvError),
    #[error("Broadcast closed waiting for event: {0:?}: {1:?}")]
    BroadcastClosed(Option<MotorEvent>, RecvError),
    #[error("Error switching to NMT state: {0:?}: {1:?}")]
    NMTSendError(NmtState, SendError<NmtState>),
    #[error("Error switching to NMT state: {0:?}")]
    NMTSwitchError(NmtState),
    #[error("Error sending new setpoint do setpoint manager: {0:?}: {1:?}")]
    NewSetpointSendError(Setpoint, SendError<Setpoint>),
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
    #[error("Error in inter-task communication: {0}")]
    InterTaskCommunicationError(String),
    #[error("Error switching to mode {0:?}")]
    ModeSwitchError(watch::error::SendError<SetpointManagerModeTypes>),
    #[error("Attempting to write setpoint: {0:?} in wrong OperationMode: {1:?}")]
    PdoWrongSetpoint(Setpoint, OperationMode),
    #[error("Invalid mapping: {0:?}")]
    PdoWrongMapping(PdoMapping),
}

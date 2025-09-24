use oze_canopen::error::CoError;
use thiserror::Error;

use crate::state::Cia402State;

#[derive(Debug, Error)]
pub enum DriveError {
    #[error("invalid state transition from {0:?} to {1:?}")]
    InvalidTransition(Cia402State, Cia402State),
    #[error("CANopen communication error: {0:?}")]
    CanOpen(CoError),
    #[error("Invalid conversion of {0:?} into integer")]
    Conversion(Vec<u8>),
}

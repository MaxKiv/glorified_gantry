use oze_canopen::error::CoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MotorError {
    #[error("Canopen error: {0:?}")]
    CanOpen(CoError),
    #[error("Error constructing SDO Client")]
    SdoClientConstruction,
}

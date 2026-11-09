use crate::canopen::CanOpenError;

#[derive(Debug, thiserror::Error)]
pub enum MotorError {
    #[error("Unable to remap PDO: {0:?}")]
    PdoRemapping(CanOpenError),
    #[error("Unable to default parametrise motor: {0:?}")]
    UnableToDefaultParametrise(CanOpenError),
}

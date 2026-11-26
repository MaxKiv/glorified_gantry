use crate::canopen::CanOpenError;

#[derive(Debug, thiserror::Error)]
pub enum RtError {
    #[error("Timer error")]
    Timer,
    #[error("Polling error")]
    Poll,
    #[error("Invalid Motor")]
    InvalidMotor,
    #[error("Unable to start motor")]
    Startup,
    #[error("Canopen Error: {0:?}")]
    CanOpen(CanOpenError),
}

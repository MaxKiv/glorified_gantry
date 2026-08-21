use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {0:?}")]
    IoError(io::Error),
    #[error("Unable to lock mutex")]
    LockError,
    #[error("RingBuffer is full")]
    Full,
}

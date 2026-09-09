#[derive(Debug, thiserror::Error)]
pub enum FifoError<T> {
    #[error("Fifo is full")]
    Full(T),
    #[error("Fifo is empty")]
    Empty,
}

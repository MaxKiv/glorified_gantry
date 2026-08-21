#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Fifo is full")]
    Full,
    #[error("Fifo is empty")]
    Empty,
}

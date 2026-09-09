use crate::{frontend::GantryCommand, rt::cmd::RtCommand, spsc::ringbuffer::SpScRingBuffer};

/// CMD Queue as SPSC ring buffer
pub struct CommandQueue<const N: usize> {
    inner: SpScRingBuffer<GantryCommand, N>,
}

impl<const N: usize> CommandQueue<N> {
    /// Construct a new CommandQueue
    pub fn new() -> Self {
        Self {
            inner: SpScRingBuffer::new(),
        }
    }

    /// Add a command to the queue
    pub fn push(&mut self, cmd: GantryCommand) -> Result<(), GantryCommand> {
        self.inner.push(cmd)
    }

    /// Pop the latest command from the queue
    pub fn pop(&mut self) -> Option<GantryCommand> {
        self.inner.pop()
    }
}

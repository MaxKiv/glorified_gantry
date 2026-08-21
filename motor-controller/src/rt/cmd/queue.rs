use crate::{rt::cmd::RtCommand, spsc::ringbuffer::SpScRingBuffer};

/// CMD Queue as SPSC ring buffer
pub struct CommandQueue<const N: usize> {
    inner: SpScRingBuffer<RtCommand, N>,
}

impl<const N: usize> CommandQueue<N> {
    /// Construct a new CommandQueue
    pub fn new() -> Self {
        Self {
            inner: SpScRingBuffer::new(),
        }
    }

    /// Add a command to the queue
    pub fn push(&mut self, cmd: RtCommand) -> Result<(), RtCommand> {
        self.inner.push(cmd)
    }

    /// Pop the latest command from the queue
    pub fn pop(&mut self) -> Option<RtCommand> {
        self.inner.pop()
    }
}

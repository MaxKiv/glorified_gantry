use std::{
    cell::UnsafeCell,
    fmt::Display,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::rt::cmd::{RtCommand, channel::CmdChannelError};

/// CMD Ring buffer
pub struct CommandQueue<const N: usize> {
    write_idx: AtomicUsize,
    read_idx: AtomicUsize,
    buffer: [UnsafeCell<MaybeUninit<RtCommand>>; N],
}

impl<const N: usize> CommandQueue<N> {
    pub fn new() -> Self {
        Self {
            write_idx: AtomicUsize::new(0),
            read_idx: AtomicUsize::new(0),
            buffer: [const { UnsafeCell::new(MaybeUninit::uninit()) }; N],
        }
    }

    //   x
    // 1 2 3 | 4
    // 1 2 3 | 4
    // |

    // Empty
    //   x
    // 1 2 3 | 4    w = 2
    // 1 2 3 | 4    r = 2
    //   |          w = r

    // Full
    //   x       x
    // 1 2 3 | 4 5  w = 5
    // 1 2 3 | 4 5  r = 2
    //   ^          w - r = N

    /// Add a command to the queue
    pub fn push(&mut self, cmd: RtCommand) -> Result<RtCommand, CmdChannelError> {
        tracing::debug!("CommandQueue::push({:?})", cmd);

        let write = self.write_idx.load(Ordering::Relaxed);
        let read = self.read_idx.load(Ordering::Acquire);

        // Check for full
        if write - read == N {
            return Err(CmdChannelError::Full);
        }

        let idx = write % N;

        unsafe {
            (*self.buffer[idx].get()).write(cmd);
        }

        // Update write ptr
        self.write_idx.store(write + 1, Ordering::Release);

        tracing::debug!("{}", self);

        Ok(cmd)
    }

    pub fn pop(&mut self) -> Option<RtCommand> {
        let write = self.write_idx.load(Ordering::Relaxed);
        let read = self.read_idx.load(Ordering::Acquire);

        // Check for empty
        if write == read {
            // Empty
            return None;
        }

        // Not Empty; read
        let idx = read % N;
        let out = unsafe { (*self.buffer[idx].get()).assume_init() };

        // Update read ptr
        self.read_idx.store(read + 1, Ordering::Release);

        tracing::debug!("{}", self);

        Some(out)
    }
}

impl<const N: usize> Display for CommandQueue<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write = self.write_idx.load(Ordering::Relaxed);
        let read = self.read_idx.load(Ordering::Acquire);

        const TAB: &str = "\t";

        // show write ptr
        write!(f, "\n")?;
        for _ in 0..write {
            write!(f, "w{}", TAB)?;
        }
        write!(f, "\n")?;

        // show queue
        write!(f, "(",)?;
        for i in 0..(N - 1) {
            let cmd = unsafe { (*self.buffer[i].get()).assume_init() };
            write!(f, "{:?},\t", cmd)?;
        }
        let cmd = unsafe { (*self.buffer[N - 1].get()).assume_init() };
        write!(f, "{:?})", cmd)?;

        // show read ptr
        write!(f, "\n")?;
        for _ in 0..read {
            write!(f, "r{}", TAB)?;
        }
        write!(f, "\n")?;

        Ok(())
    }
}

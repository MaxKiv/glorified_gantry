use std::{fmt::Display, sync::atomic::Ordering};

use crate::spsc::ringbuffer::SpScRingBuffer;

impl<T, const N: usize> Display for SpScRingBuffer<T, N>
where
    T: std::fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write = self.write_idx.load(Ordering::Relaxed);
        let read = self.read_idx.load(Ordering::Acquire);

        const TAB: &str = "\t";

        // show write ptr
        write!(f, "\n")?;
        for _ in 0..(write % N) {
            write!(f, "w{}", TAB)?;
        }
        write!(f, "\n")?;

        // show queue
        write!(f, "(",)?;
        for i in 0..(N - 1) {
            let cmd = unsafe { (*self.buffer[i].get()).assume_init_ref() };
            write!(f, "{:?},\t", cmd)?;
        }
        let cmd = unsafe { (*self.buffer[N - 1].get()).assume_init_ref() };
        write!(f, "{:?})", cmd)?;

        // show read ptr
        write!(f, "\n")?;
        for _ in 0..(read % N) {
            write!(f, "r{}", TAB)?;
        }
        write!(f, "\n")?;

        Ok(())
    }
}

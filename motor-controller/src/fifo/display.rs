use std::{fmt::Display, sync::atomic::Ordering};

use crate::fifo::Fifo;

impl<T, const N: usize> Display for Fifo<T, N>
where
    T: std::fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write = self.write;
        let read = self.read;

        const TAB: &str = "\t\t";

        // show write ptr
        write!(f, "\n")?;
        for _ in 0..(write % N) {
            write!(f, "w{}", TAB)?;
        }
        write!(f, "\n")?;

        // show queue
        write!(f, "(",)?;
        for i in 0..(N - 1) {
            let cmd = unsafe { self.buff[i].assume_init_ref() };
            write!(f, "{:?},\t", cmd)?;
        }
        let cmd = unsafe { self.buff[N - 1].assume_init_ref() };
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

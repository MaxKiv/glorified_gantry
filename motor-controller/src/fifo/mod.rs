pub mod display;
pub mod error;

use std::mem::MaybeUninit;

use crate::fifo::error::FifoError;

/// A single thread FIFO impl
#[derive(Debug)]
pub struct Fifo<T, const N: usize> {
    buff: [MaybeUninit<T>; N],
    write: usize,
    read: usize,
}

impl<T, const N: usize> Fifo<T, N> {
    pub fn new() -> Self {
        Self {
            buff: [const { MaybeUninit::uninit() }; N],
            write: 0,
            read: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.read == self.write
    }

    pub fn push(&mut self, item: T) -> Result<usize, FifoError<T>> {
        // Check full
        if self.write - self.read == N {
            return Err(FifoError::Full(item));
        }

        self.buff[self.write % N].write(item);

        self.write += 1;

        Ok(self.write)
    }

    pub fn pop(&mut self) -> Result<T, FifoError<T>> {
        if self.is_empty() {
            return Err(FifoError::Empty);
        }

        let out = unsafe { self.buff[self.read % N].assume_init_read() };

        self.read += 1;

        Ok(out)
    }
}

impl<T, const N: usize> Drop for Fifo<T, N> {
    fn drop(&mut self) {
        while let Ok(_) = self.pop() {} // Dropping MaybeUninit<T> doesn't drop T, pop() does
    }
}

#[cfg(test)]
mod common {

    use crate::utils::setup_tracing_subscriber;

    use super::*;

    const TRIES: usize = 1000;

    #[test]
    fn fifo_basics() -> anyhow::Result<()> {
        setup_tracing_subscriber();

        let mut fifo = Fifo::<usize, TRIES>::new();

        let items: [usize; TRIES] = core::array::from_fn(|i| i);

        tracing::trace!("items: {:?}", items);

        for item in items {
            fifo.push(item)
                .expect(&format!("should be able to push item: {} into fifo", item));
        }

        fifo.push(0)
            .expect_err("Should be unable to push another element into fifo");

        for item in items.iter() {
            let x = fifo
                .pop()
                .expect(&format!("Should be able to pop item: {}", item));

            tracing::trace!("{x} - {item}");

            if x != *item {
                panic!("Ordering wrong: {} != {}\n{:?}", x, item, fifo);
            }
        }

        Ok(())
    }
}

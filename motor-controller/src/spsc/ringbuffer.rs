use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Single Producer Single Consumer ring buffer
pub struct SpScRingBuffer<T, const N: usize> {
    pub(in crate::spsc) write_idx: AtomicUsize,
    pub(in crate::spsc) read_idx: AtomicUsize,
    pub(in crate::spsc) buffer: [UnsafeCell<MaybeUninit<T>>; N],
}

impl<const N: usize, T> SpScRingBuffer<T, N>
where
    T: std::fmt::Debug + std::clone::Clone,
{
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
    /// SAFETY: Only a single producer is allowed to push
    /// Failing to uphold this invariant results in UB
    pub fn push(&self, item: T) -> Result<(), T> {
        tracing::debug!("SpScRingBuffer::push({:?})", item);

        let write = self.write_idx.load(Ordering::Relaxed);
        let read = self.read_idx.load(Ordering::Acquire);

        // Check for full
        if write - read == N {
            return Err(item);
        }

        let idx = write % N;

        unsafe {
            (*self.buffer[idx].get()).write(item);
        }

        // Update write ptr with Ordering::Release
        self.write_idx.store(write + 1, Ordering::Release);

        tracing::debug!("{}", self);

        Ok(())
    }

    /// SAFETY: Only a single consumer is allowed to pop
    /// Failing to uphold this invariant results in UB
    pub fn pop(&self) -> Option<T> {
        tracing::debug!("SpScRingBuffer::pop()");
        let write = self.write_idx.load(Ordering::Acquire);
        let read = self.read_idx.load(Ordering::Relaxed);

        // Check for empty
        if write == read {
            // Empty
            return None;
        }

        // Not Empty; read
        let idx = read % N;
        let out = unsafe { (*self.buffer[idx].get()).assume_init_ref().clone() };

        // Update read ptr
        self.read_idx.store(read + 1, Ordering::Release);

        tracing::debug!("{}", self);

        Some(out)
    }
}

unsafe impl<T, const N: usize> Send for SpScRingBuffer<T, N> {}
unsafe impl<T, const N: usize> Sync for SpScRingBuffer<T, N> {}

#[cfg(test)]
mod common {
    use std::sync::Arc;

    use crate::{spsc::error::Error, utils::setup_tracing_subscriber};

    use super::*;

    const TRIES: usize = 100_000;

    // pushes monotonically increasing counter
    fn producer(spsc: Arc<SpScRingBuffer<usize, 10>>) -> anyhow::Result<()> {
        for i in 0..TRIES {
            loop {
                if let Ok(_) = spsc.push(i).map_err(|_| Error::Full) {
                    tracing::trace!("Pushed {}", i);
                    break;
                } else {
                    tracing::trace!("Push failed: {}", spsc);
                }
            }
        }

        Ok(())
    }

    // reads monotonically increasing counter, making sure the correct total amounts arrives in order
    fn consumer(spsc: Arc<SpScRingBuffer<usize, 10>>) -> anyhow::Result<()> {
        let mut cnt = 0usize;
        loop {
            match spsc.pop() {
                Some(n) => {
                    tracing::trace!("Read {}", n);

                    if n != cnt {
                        anyhow::bail!("consumer read: {} != cnt {}", n, cnt);
                    }

                    cnt += 1;
                }
                None => {
                    tracing::trace!("pop failed: {}", spsc);

                    if cnt >= TRIES {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    #[test]
    /// Spawns a producer and consumer thread
    /// Procucer stores monotonically increasing counter
    /// Consumer reads it and compares against internal counter
    /// Any non-monotonicity is rejected
    fn monotonic_counter_never_trashed() -> anyhow::Result<()> {
        setup_tracing_subscriber();

        // Construct SPSC ring buffer, use Arc to pass non mutable reference
        let producer_queue = Arc::new(SpScRingBuffer::new());
        let consumer_queue = producer_queue.clone();

        // spawn producer & consumer threads
        let producer = std::thread::spawn(move || producer(producer_queue));
        let consumer = std::thread::spawn(move || consumer(consumer_queue));

        // joi producer & consumer threads
        if let Err(err) = producer.join() {
            anyhow::bail!("{err:?}");
        }
        if let Err(err) = consumer.join() {
            anyhow::bail!("{err:?}");
        }

        tracing::info!("Test success!");

        Ok(())
    }
}

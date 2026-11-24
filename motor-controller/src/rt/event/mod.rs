use std::io;
use std::os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::unix::AsyncFd;

pub fn event_channel<T: Send>(capacity: usize) -> io::Result<(EventSender<T>, EventReceiver<T>)> {
    let raw = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) };
    if raw < 0 {
        return Err(io::Error::last_os_error());
    }
    let rx_fd = unsafe { OwnedFd::from_raw_fd(raw) };
    let tx_fd = rx_fd.try_clone()?;

    let (producer, consumer) = rtrb::RingBuffer::new(capacity);
    let dropped = Arc::new(AtomicU64::new(0));

    Ok((
        EventSender {
            producer,
            fd: tx_fd,
            dropped: dropped.clone(),
            pending: false,
        },
        EventReceiver {
            consumer,
            fd: rx_fd,
            dropped,
        },
    ))
    // no AsyncFd here, so no runtime needed
}

/// Not yet bound to a reactor. Send it anywhere.
pub struct EventReceiver<T> {
    consumer: rtrb::Consumer<T>,
    fd: OwnedFd,
    dropped: Arc<AtomicU64>,
}

impl<T> EventReceiver<T> {
    /// Must be called from within a tokio runtime context.
    pub fn into_async(self) -> io::Result<AsyncEventReceiver<T>> {
        Ok(AsyncEventReceiver {
            consumer: self.consumer,
            afd: AsyncFd::new(self.fd)?,
            dropped: self.dropped,
            last_reported: 0,
        })
    }
}

pub struct AsyncEventReceiver<T> {
    consumer: rtrb::Consumer<T>,
    afd: AsyncFd<OwnedFd>,
    dropped: Arc<AtomicU64>,
    last_reported: u64,
}

impl<T> AsyncEventReceiver<T> {
    /// Wait until the RT side signals, and clear the eventfd.
    /// Afterwards, drain with `try_recv()` until it returns `None`.
    pub async fn readable(&mut self) -> io::Result<()> {
        loop {
            let mut guard = self.afd.readable().await?;
            let res = guard.try_io(|fd| {
                let mut buf = 0u64;
                let n = unsafe {
                    libc::read(
                        fd.as_raw_fd(),
                        &mut buf as *mut u64 as *mut libc::c_void,
                        std::mem::size_of::<u64>(),
                    )
                };
                if n == std::mem::size_of::<u64>() as isize {
                    Ok(())
                } else {
                    Err(io::Error::last_os_error()) // EAGAIN => readiness cleared
                }
            });
            match res {
                Ok(r) => return r,
                Err(_would_block) => continue, // spurious; wait again
            }
        }
    }

    #[inline]
    pub fn try_recv(&mut self) -> Option<T> {
        self.consumer.pop().ok()
    }

    /// Number of events dropped since the last call.
    pub fn take_dropped(&mut self) -> u64 {
        let total = self.dropped.load(Ordering::Relaxed);
        let delta = total - self.last_reported;
        self.last_reported = total;
        delta
    }
}

/// RT side. Never blocks, never allocates.
pub struct EventSender<T> {
    producer: rtrb::Producer<T>,
    fd: OwnedFd,
    dropped: Arc<AtomicU64>,
    pending: bool,
}

impl<T> EventSender<T> {
    /// Enqueue an event. Drops (and counts) it if the ring is full.
    #[inline]
    pub fn push(&mut self, ev: T) {
        match self.producer.push(ev) {
            Ok(()) => self.pending = true,
            Err(_) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Wake the async side. Call once per control cycle, after pushes.
    #[inline]
    pub fn notify(&mut self) {
        if !self.pending {
            return;
        }
        self.pending = false;
        let one = 1u64;
        // Non-blocking; EAGAIN only if the counter would overflow, which
        // means a wakeup is already pending. Safe to ignore.
        let _ = unsafe {
            libc::write(
                self.fd.as_raw_fd(),
                &one as *const u64 as *const libc::c_void,
                std::mem::size_of::<u64>(),
            )
        };
    }
}

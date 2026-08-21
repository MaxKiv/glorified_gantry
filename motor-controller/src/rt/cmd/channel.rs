use std::io;
use std::os::fd::RawFd;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::rt::cmd::RtCommand;
use crate::rt::cmd::queue::CommandQueue;
use crate::spsc::error::Error;

pub struct CmdChannel<const N: usize> {
    rx: CmdReceiver<N>,
    tx: CmdSender<N>,
}

pub struct CmdReceiver<const N: usize> {
    fd: RawFd,
    queue: Arc<Mutex<CommandQueue<N>>>,
}

pub struct CmdSender<const N: usize> {
    fd: RawFd,
    queue: Arc<Mutex<CommandQueue<N>>>,
}

impl<const N: usize> CmdChannel<N> {
    pub fn new() -> std::io::Result<(CmdSender<N>, CmdReceiver<N>)> {
        let queue = Arc::new(Mutex::new(CommandQueue::<N>::new()));
        let tx = CmdSender::new(queue)?;
        let rx = tx.duplicate();

        match rx {
            Ok(rx) => Ok((tx, rx)),
            Err(err) => {
                unsafe { libc::close(tx.fd()) };
                Err(err)
            }
        }
    }
}

impl<'a, const N: usize> CmdReceiver<N> {
    pub fn fd(&self) -> RawFd {
        self.fd
    }

    fn read_eventfd(&self) -> io::Result<u64> {
        // Check eventfd for number of enqueued cmds
        let mut work_required = 0u64;
        let result = unsafe {
            libc::read(
                self.fd,
                &mut work_required as *mut u64 as *mut libc::c_void,
                std::mem::size_of::<u64>(),
            )
        };

        if result != std::mem::size_of::<u64>() as isize {
            return Err(io::Error::last_os_error());
        }

        tracing::debug!("CmdReceiver::read_eventfd() - {}", work_required);
        Ok(work_required)
    }

    pub fn drain(&'a mut self) -> Result<CmdDrain<'a, N>, Error> {
        // Reset eventfd
        let _ = self.read_eventfd().map_err(|err| Error::IoError(err))?;

        // Lock mutex to guarantee RT gets priority
        // NOTE: this could block current thread, maybe we should attempt to block first, and only
        // drain eventfd later? Or provide a timeout?

        tracing::debug!("CmdReceiver::drain() - locking");
        let queue_lock = self.queue.lock().map_err(|_| Error::LockError)?;
        Ok(CmdDrain { queue_lock })
    }
}

pub struct CmdDrain<'a, const N: usize> {
    queue_lock: MutexGuard<'a, CommandQueue<N>>,
}

impl<'a, const N: usize> Iterator for CmdDrain<'a, N> {
    type Item = RtCommand;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue_lock.pop()
    }
}

impl<const N: usize> CmdSender<N> {
    pub fn new(queue: Arc<Mutex<CommandQueue<N>>>) -> io::Result<Self> {
        let fd = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) };

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self { fd, queue })
    }

    pub fn duplicate(&self) -> io::Result<CmdReceiver<N>> {
        let dup_fd = unsafe { libc::dup(self.fd) };
        let queue = self.queue.clone();

        if dup_fd < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(CmdReceiver { fd: dup_fd, queue })
    }

    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn send(&self, cmd: RtCommand) -> Result<(), Error> {
        // Block on Enqueue cmd
        loop {
            if let Ok(_) = self.queue.lock().map_err(|_| Error::LockError)?.push(cmd) {
                break;
            }
        }
        tracing::debug!("CmdReceiver::send() - enqueued cmd {:?}", cmd);

        // Notify through eventfd
        let result = unsafe {
            libc::write(
                self.fd,
                &1u64 as *const u64 as *const libc::c_void,
                std::mem::size_of::<u64>(),
            )
        };

        if result != std::mem::size_of::<u64>() as isize {
            return Err(Error::IoError(io::Error::last_os_error()));
        }

        Ok(())
    }
}

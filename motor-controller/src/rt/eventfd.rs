use core::io;
use std::os::fd::RawFd;

pub struct EventFdChannel {
    rx: EventFdRx,
    tx: EventFdTx,
}

pub struct EventFdRx {
    fd: RawFd,
}

pub struct EventFdTx {
    fd: RawFd,
}

impl EventFdChannel {
    pub fn new() -> io::Result<(EventFdTx, EventFdRx)> {
        let tx = EventFdTx::new()?;
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

impl EventFdRx {
    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn notify(&self) -> io::Result<()> {
        let value = 1u64;

        let result = unsafe {
            libc::write(
                self.fd,
                &value as *const u64 as *const libc::c_void,
                std::mem::size_of::<u64>(),
            )
        };

        if result != std::mem::size_of::<u64>() as isize {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn read(&self) -> io::Result<u64> {
        let mut out = 0u64;

        let result = unsafe {
            libc::read(
                self.fd,
                &mut out as *mut u64 as *mut libc::c_void,
                std::mem::size_of::<u64>(),
            )
        };

        if result != std::mem::size_of::<u64>() as isize {
            return Err(io::Error::last_os_error());
        }

        Ok(out)
    }
}

impl EventFdTx {
    pub fn new() -> io::Result<Self> {
        let fd = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) };

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self { fd })
    }

    pub fn duplicate(&self) -> io::Result<EventFdRx> {
        let dup_fd = unsafe { libc::dup(self.fd) };

        if dup_fd < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(EventFdRx { fd: dup_fd })
    }

    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn notify(&self) -> io::Result<()> {
        let value = 1u64;

        let result = unsafe {
            libc::write(
                self.fd,
                &value as *const u64 as *const libc::c_void,
                std::mem::size_of::<u64>(),
            )
        };

        if result != std::mem::size_of::<u64>() as isize {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

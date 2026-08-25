use std::{io, os::fd::RawFd};

pub struct EventFd {
    fd: RawFd,
}

impl EventFd {
    pub fn try_new(fd: RawFd) -> io::Result<Self> {
        let fd = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) };

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self { fd })
    }
}

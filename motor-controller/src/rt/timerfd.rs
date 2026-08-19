use core::io;
use std::{os::fd::RawFd, time::Duration};

/// A file descriptor backed timer
/// Unix only, using licb::timerfd_create
pub struct TimerFd {
    fd: RawFd,
}

impl TimerFd {
    pub fn from_period(period: Duration) -> std::io::Result<Self> {
        let fd = unsafe { libc::timerfd_create(libc::CLOCK_MONOTONIC, libc::TFD_CLOEXEC) };

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let period = period.as_nanos() as i64;
        let timer = libc::itimerspec {
            it_interval: libc::timespec {
                tv_sec: 0,
                tv_nsec: period,
            },
            it_value: libc::timespec {
                tv_sec: 0,
                tv_nsec: period,
            },
        };

        let result = unsafe {
            libc::timerfd_settime(fd, libc::TFD_TIMER_ABSTIME, &timer, std::ptr::null_mut())
        };

        if result < 0 {
            let error = io::Error::last_os_error();

            unsafe { libc::close(fd) };

            return Err(error);
        }

        Ok(Self { fd })
    }

    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn expirations(&self) -> io::Result<u64> {
        let mut expirations = 0u64;

        let err = unsafe {
            libc::read(
                self.fd,
                &mut expirations as *mut u64 as *mut libc::c_void,
                core::mem::size_of::<u64>(),
            )
        };

        if err != core::mem::size_of::<u64>() as isize {
            return Err(io::Error::last_os_error());
        }

        Ok(expirations)
    }
}

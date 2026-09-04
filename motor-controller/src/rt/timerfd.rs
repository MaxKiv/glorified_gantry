use std::{io, os::fd::RawFd, time::Duration};

#[derive(PartialEq)]
pub enum TimerType {
    Absolute,
    Relative,
}

/// A file descriptor backed timer
/// Unix only, using licb::timerfd_create
pub struct TimerFd {
    name: &'static str,
    /// Period of clock
    period: Duration,
    /// Period of clock
    period_ns: i64,
    /// Precomputed timespec
    timerspec: libc::itimerspec,
    /// Is period relative to system time, or moment clock started?
    timer_type: TimerType,
    /// Raw file descriptor
    fd: RawFd,
}

impl TimerFd {
    pub fn from_period_monotonic(period: Duration, timer_type: TimerType) -> std::io::Result<Self> {
        let fd = unsafe {
            libc::timerfd_create(
                libc::CLOCK_MONOTONIC,
                libc::TFD_CLOEXEC | libc::TFD_NONBLOCK,
            )
        };

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let timespec = libc::itimerspec {
            it_interval: libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            it_value: libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
        };

        Ok(Self {
            fd,
            period,
            period_ns: period.as_nanos() as i64,
            timer_type,
            timerspec: timespec,
        })
    }

    fn timerfd_settime(&mut self) -> io::Result<()> {
        let flags = if self.timer_type == TimerType::Absolute {
            libc::TFD_TIMER_ABSTIME
        } else {
            0
        };
        let result = unsafe {
            libc::timerfd_settime(self.fd(), flags, &self.timerspec, std::ptr::null_mut())
        };

        if result < 0 {
            let error = io::Error::last_os_error();

            unsafe { libc::close(self.fd()) };

            return Err(error);
        }

        Ok(())
    }

    /// start timer
    pub fn arm(&mut self) -> io::Result<()> {
        self.timerspec.it_interval.tv_nsec = self.period_ns;
        self.timerspec.it_value.tv_nsec = self.period_ns;
        self.timerfd_settime()
    }

    /// start oneshot timer
    pub fn arm_once(&mut self) -> io::Result<()> {
        self.timerspec.it_value.tv_nsec = self.period_ns;
        self.timerfd_settime()
    }

    /// Stop timer
    pub fn disarm(&mut self) -> io::Result<()> {
        self.timerspec.it_value.tv_nsec = self.period_ns;
        self.timerfd_settime()
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

    pub fn reset(&self) -> io::Result<u64> {
        let expirations = self.expirations()?;

        // Did the timer overrun?
        if expirations != 1 {
            tracing::error!("RT: {} timer {} overruns", self.name, expirations);
        }

        Ok(expirations)
    }
}

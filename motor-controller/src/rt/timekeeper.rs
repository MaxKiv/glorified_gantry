use libc::timespec;

use crate::consts::RT_CYCLE_PERIOD;

#[derive(Debug, Clone, Copy)]
pub struct CycleTiming {
    cycle: u64,
    expected_ns: u64,
    actual_ns: u64,
    jitter_ns: i64,
    execution_ns: u64,
}

pub struct TimeKeeper {
    cycle: u64,
    start_cycle: libc::timespec,
    end_sync: libc::timespec,
    prev_start: Option<libc::timespec>,
}

impl TimeKeeper {
    pub fn new() -> Self {
        Self {
            cycle: 0u64,
            start_cycle: timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            end_sync: timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            prev_start: None,
        }
    }

    /// Produces timing stats from the current timekeeper stats
    pub fn get_cycle_timing(&self) -> CycleTiming {
        let execution_ns = 1_000_000_000i64 * (self.end_sync.tv_sec - self.start_cycle.tv_sec)
            + (self.end_sync.tv_nsec - self.start_cycle.tv_nsec);

        let actual_ns = if let Some(prev_start) = &self.prev_start {
            (1_000_000_000i64 * (self.start_cycle.tv_sec - prev_start.tv_sec)
                + (self.start_cycle.tv_nsec - prev_start.tv_nsec)) as u64
        } else {
            0u64
        };

        let expected_ns = RT_CYCLE_PERIOD.as_nanos() as u64;
        let jitter_ns = actual_ns as i64 - expected_ns as i64;

        let cycle_timing = CycleTiming {
            cycle: self.cycle,
            expected_ns,
            actual_ns,
            jitter_ns,
            execution_ns: execution_ns as u64,
        };

        cycle_timing
    }

    pub fn start_new_cycle(&mut self) {
        TimeKeeper::time(&mut self.start_cycle)
    }

    fn time_end_sync(&mut self) {
        TimeKeeper::time(&mut self.end_sync)
    }

    pub fn end_cycle(&mut self) -> CycleTiming {
        self.time_end_sync();
        let cycle_timing = self.get_cycle_timing();
        self.prev_start = Some(self.start_cycle);
        self.cycle += 1;
        cycle_timing
    }

    fn time(timespec: &mut timespec) {
        if unsafe {
            libc::clock_gettime(
                libc::CLOCK_MONOTONIC,
                (timespec) as *const _ as *mut libc::timespec,
            )
        } != 0
        {
            let error = std::io::Error::last_os_error();
            tracing::error!("unable to time: {:?}", error);
        }
    }
}

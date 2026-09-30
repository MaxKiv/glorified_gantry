use libc::timespec;
use tracing::error;

use crate::{consts::RT_CONFIG, rt::engine::cycle_rx::CyclePhase};

#[derive(Debug, Clone, Copy)]
pub struct CycleTiming {
    cycle: u64,
    cycle_expected_ns: u64,
    cycle_actual_ns: u64,
    sync_to_sync_jitter_ns: i64,
    feedback_duration_ns: u64,
    cycle_execution_ns: u64,
}

pub struct TimeKeeper {
    state: CyclePhase,
    start_cycle: libc::timespec,
    start_feedback: libc::timespec,
    end_feedback: libc::timespec,
    sent_motor_setpoints: libc::timespec,
    start_sdo_window: libc::timespec,
    end_cycle: libc::timespec,
    prev_start: Option<libc::timespec>,
}

impl TimeKeeper {
    pub fn new() -> Self {
        Self {
            state: CyclePhase::SendingSync,
            start_cycle: timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            sent_motor_setpoints: timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            start_feedback: timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            end_feedback: timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            start_sdo_window: timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            end_cycle: timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            prev_start: None,
        }
    }

    /// Produces timing stats from the current timekeeper stats
    pub fn get_cycle_timing(&self, cycle: u64) -> CycleTiming {
        let execution_ns = 1_000_000_000i64
            * (self.sent_motor_setpoints.tv_sec - self.start_cycle.tv_sec)
            + (self.sent_motor_setpoints.tv_nsec - self.start_cycle.tv_nsec);

        let actual_ns = if let Some(prev_start) = &self.prev_start {
            (1_000_000_000i64 * (self.start_cycle.tv_sec - prev_start.tv_sec)
                + (self.start_cycle.tv_nsec - prev_start.tv_nsec)) as u64
        } else {
            0u64
        };

        let expected_ns = RT_CONFIG.cycle_period.as_nanos() as u64;
        let jitter_ns = actual_ns as i64 - expected_ns as i64;

        let feedback_duration_ns =
            (1_000_000_000i64 * (self.end_feedback.tv_sec - self.start_feedback.tv_sec)
                + (self.end_feedback.tv_nsec - self.start_feedback.tv_nsec)) as u64;

        let cycle_timing = CycleTiming {
            cycle,
            cycle_expected_ns: expected_ns,
            cycle_actual_ns: actual_ns,
            sync_to_sync_jitter_ns: jitter_ns,
            cycle_execution_ns: execution_ns as u64,
            feedback_duration_ns,
        };

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

    pub fn on_sync_cycle_start(&mut self) {
        TimeKeeper::time(&mut self.start_cycle)
    }

    pub fn on_motor_setpoints_sent(&mut self) {
        TimeKeeper::time(&mut self.sent_motor_setpoints)
    }

    pub fn on_sdo_window_start(&mut self) {
        TimeKeeper::time(&mut self.start_sdo_window)
    }

    fn time_cycle_end(&mut self) {
        TimeKeeper::time(&mut self.end_cycle);
        self.prev_start = Some(self.start_cycle);
    }

    pub fn on_cycle_end(&mut self) {
        self.time_cycle_end();
    }

    pub fn on_start_waiting_for_feedback(&mut self) {
        TimeKeeper::time(&mut self.start_feedback)
    }

    pub fn on_receive_feedback(&mut self) {
        TimeKeeper::time(&mut self.end_feedback)
    }
}

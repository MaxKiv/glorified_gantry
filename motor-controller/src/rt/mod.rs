pub mod cmd;
pub mod eventfd;
pub mod timekeeper;
pub mod timerfd;

use core::io;
use std::{
    os::fd::{AsFd, AsRawFd},
    time::Duration,
};

use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Frame, Socket};
use tracing::{error, info, trace};

use crate::{
    cw::ControlWord,
    rt::{
        eventfd::EventFdRx,
        timekeeper::{CycleTiming, TimeKeeper},
        timerfd::TimerFd,
    },
};

// Tunable constants
const RT_CYCLE_TIME: Duration = Duration::from_millis(1);
const MAX_CAN_RX_PER_POLL: usize = 10; // TODO: determine this, although it seems can.read_frame() always returns io::error::WouldBlock after 1 call

// Other constants
const SYNC: usize = 0;
const CAN_RX: usize = 1;
const CMD_RX: usize = 2;

pub struct RealTimeSetpoint {
    generation: u64,
    controlword: ControlWord,
    target: i32,
    // OR?
    // target: MotorSetpoint(ProfilePosition(10))
}

pub struct RealTimeConfig {
    // pdo_mapping: PdoMapping,
}

pub struct RealTimeComms {
    // event_rx: RT_API::Receiver<RealTimeFeedback>,
}

#[derive(Debug, thiserror::Error)]
pub enum RtErrors {
    #[error("other")]
    Other,
}

pub struct MotorFeedback {}

struct RealTimeFeedback<const N: usize> {
    cycle: u64,
    motors: [MotorFeedback; N],
    timing: CycleTiming,
    errors: RtErrors,
    skew: Option<f64>,
}

pub struct RealTimeEngine {
    can_interface: String,
    cmd_rx: EventFdRx,
}

impl RealTimeEngine {
    pub fn start(can_interface: String, cmd_rx: EventFdRx) -> std::thread::JoinHandle<()> {
        let rt = Self {
            can_interface,
            cmd_rx,
        };
        std::thread::spawn(move || rt.run())
    }

    fn run(&self) {
        info!("RT Thread started");

        let mut timekeeper = TimeKeeper::new();

        let sync = CanFrame::from_raw_id(0x080, &[]).expect("failed to construct SYNC frame");

        let timer = TimerFd::from_period(RT_CYCLE_TIME).expect("Failed to create RT timer");

        let mut can = CanSocket::open(&self.can_interface).expect("Unable to open CAN interface");
        can.set_nonblocking(true)
            .expect("Unable to set CAN socket nonblocking");

        let mut poll_fds = [
            libc::pollfd {
                fd: timer.fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: can.as_fd().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: self.cmd_rx.fd(),
                events: libc::POLLIN,
                revents: 0,
            },
        ];

        loop {
            // Poll FDs
            let result =
                unsafe { libc::poll(poll_fds.as_mut_ptr(), poll_fds.len() as libc::nfds_t, -1) };
            if result < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                panic!("poll failed: {error}");
            }

            // Service events
            if poll_fds[CMD_RX].revents & libc::POLLIN != 0 {
                self.process_cmd_rx();
            }
            if poll_fds[CAN_RX].revents & libc::POLLIN != 0 {
                self.process_can_rx(&mut can);
            }
            if poll_fds[SYNC].revents & libc::POLLIN != 0 {
                self.sync_cycle(&can, &sync, &timer, &mut timekeeper);
            }
            info!("looping");
        }
    }

    fn sync_cycle(
        &self,
        can: &CanSocket,
        sync: &CanFrame,
        timer: &TimerFd,
        timekeeper: &mut TimeKeeper,
    ) {
        // Time cycle
        timekeeper.start_new_cycle();

        // Check timer expirations
        let expirations = timer.expirations().expect("TimerFD read failed");

        // Check if we missed any
        if expirations != 1 {
            error!("RT overrun: {} timer expirations", expirations);
        }

        // Write SYNC
        can.write_frame(sync).expect("Unable to write SYNC");

        // Bookkeeping
        let cycle_timing = timekeeper.end_cycle();

        info!("{:?} - SYNC", cycle_timing);
    }

    fn process_can_rx(&self, can: &CanSocket) {
        for _ in 0..MAX_CAN_RX_PER_POLL {
            match can.read_frame() {
                Ok(frame) => {
                    info!(
                        "CAN RX id={:#x} data={:?}",
                        frame.raw_id(),
                        &frame.data()[..frame.data().len()]
                    );
                }

                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    trace!("would block");
                    break;
                }

                Err(error) => {
                    error!("CAN RX error: {error}");
                    break;
                }
            }
        }
    }

    fn process_cmd_rx(&self) {
        // Drain command queue
        match self.cmd_rx.read() {
            Ok(count) => {
                info!("RT command notification: {count}");
            }

            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                // Nothing left to consume.
            }

            Err(error) => {
                error!("eventfd read failed: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    use crate::rt::eventfd::EventFdChannel;

    use super::*;

    #[test]
    fn rt() -> anyhow::Result<()> {
        setup_tracing_subscriber();

        let (cmd_tx, cmd_rx) = EventFdChannel::new()?;

        info!("Starting rt engine");
        let rt_engine = RealTimeEngine::start(String::from("can0"), cmd_rx);

        let tokio_rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        tokio_rt.block_on(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                cmd_tx.notify().expect("failed to notify RT");
                cmd_tx.notify().expect("failed to notify RT");
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                cmd_tx.notify().expect("failed to notify RT");
                info!("tokio looping");
            }
        });

        info!("Joining rt engine");
        if let Err(err) = rt_engine.join() {
            anyhow::bail!("failed to join rt engine thread: {err:?}");
        }

        Ok(())
    }

    fn setup_tracing_subscriber() {
        // a builder for `FmtSubscriber`.
        let subscriber = FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_max_level(Level::INFO)
            // completes the builder.
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }
}

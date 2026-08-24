use std::{
    os::fd::{AsFd, AsRawFd},
    thread::JoinHandle,
};

use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Frame, Socket};
use tracing::{error, info, trace};

use crate::{
    consts::RT_CONFIG,
    fifo::Fifo,
    rt::{
        RtError,
        cmd::{RtCommand, channel::CmdReceiver},
        timekeeper::TimeKeeper,
        timerfd::{TimerFd, TimerType},
    },
};

const SYNC: usize = 0;
const FEEDBACK: usize = 1;
const CAN_RX: usize = 2;
const CMD_RX: usize = 3;
const CMD_CHANNEL_SIZE: usize = RT_CONFIG.cmd_channel_size;
const CMD_QUEUE_SIZE: usize = RT_CONFIG.cmd_channel_size;

#[derive(Default)]
enum RtState {
    #[default]
    Idle,
    SingleCycle,
    Cyclic,
    Reconfiguring,
    Faulted,
    Shutdown,
}

pub struct RtEngine {
    can_interface: String,
    cmd_channel_rx: CmdReceiver<CMD_CHANNEL_SIZE>,
    cmd_queue: Fifo<RtCommand, CMD_QUEUE_SIZE>,
    state: RtState,
    sync_frame: CanFrame,
}

impl RtEngine {
    pub fn start(
        can_interface: String,
        cmd_rx: CmdReceiver<CMD_CHANNEL_SIZE>,
    ) -> JoinHandle<Result<(), RtError>> {
        let sync: CanFrame =
            CanFrame::from_raw_id(0x080, &[]).expect("failed to construct SYNC frame");

        let mut rt = Self {
            can_interface,
            cmd_channel_rx: cmd_rx,
            state: RtState::default(),
            cmd_queue: Fifo::<RtCommand, CMD_QUEUE_SIZE>::new(),
            sync_frame: sync,
        };

        // Spawn RT engine thread
        std::thread::spawn(move || rt.run())
    }

    fn run(&mut self) -> Result<(), RtError> {
        info!("RT Thread started");

        let mut timekeeper = TimeKeeper::new();

        // Constructs a new sync cycle timer
        // implemented as absolute monotonic clock
        let mut sync_timer =
            TimerFd::from_period_monotonic(RT_CONFIG.cycle_period, TimerType::Absolute)
                .expect("Failed to create RT timer");

        // Construccts a new feedback timer
        // implemented as relative monotonic clock
        let mut feedback_timer =
            TimerFd::from_period_monotonic(RT_CONFIG.feedback_period, TimerType::Relative)
                .expect("Failed to create RT timer");

        let mut can = CanSocket::open(&self.can_interface).expect("Unable to open CAN interface");
        can.set_nonblocking(false)
            .expect("Unable to set CAN socket nonblocking");

        // Construct poll FDs
        let mut poll_fds = [
            libc::pollfd {
                fd: sync_timer.fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: feedback_timer.fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: can.as_fd().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: self.cmd_channel_rx.fd(),
                events: libc::POLLIN,
                revents: 0,
            },
        ];

        // Start SYNC timer
        sync_timer.arm().map_err(|_| RtError::Timer)?;
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
            if Self::cmd_received(&poll_fds) {
                self.process_cmd_rx();
            }
            if Self::feedback_time_elapsed(&poll_fds) {
                self.feedback_timer_elapsed();
            }
            if Self::can_frame_received(&poll_fds) {
                self.process_can_rx(&mut can);
            }
            if Self::sync_timer_elapsed(&poll_fds) {
                self.start_sync_cycle(&can, &sync_timer, &mut feedback_timer, &mut timekeeper)?;
            }

            info!("RT engine looping");
        }
    }

    fn feedback_time_elapsed(poll_fds: &[libc::pollfd]) -> bool {
        poll_fds[FEEDBACK].revents & libc::POLLIN != 0
    }

    fn cmd_received(poll_fds: &[libc::pollfd]) -> bool {
        poll_fds[CMD_RX].revents & libc::POLLIN != 0
    }

    fn can_frame_received(poll_fds: &[libc::pollfd]) -> bool {
        poll_fds[CAN_RX].revents & libc::POLLIN != 0
    }

    fn sync_timer_elapsed(poll_fds: &[libc::pollfd]) -> bool {
        poll_fds[SYNC].revents & libc::POLLIN != 0
    }

    fn start_sync_cycle(
        &mut self,
        can: &CanSocket,
        sync_timer: &TimerFd,
        feedback_timer: &mut TimerFd,
        timekeeper: &mut TimeKeeper,
    ) -> Result<(), RtError> {
        // Time cycle
        timekeeper.start_new_cycle();

        // Check timer expirations
        let expirations = sync_timer.expirations().expect("TimerFD read failed");

        // Check if we missed any
        if expirations != 1 {
            error!("RT overrun: {} timer expirations", expirations);
        }

        // Check cmd queue
        if !self.cmd_queue.is_empty() {
            let cmd = self
                .cmd_queue
                .pop()
                .expect("no cmd in cmd_queue after !is_empty()");
            info!("self.transition_to({:?})", cmd);
            self.transition_to(cmd);
        }

        // Write SYNC
        self.send_sync(&can);

        // Setup feedback timer
        feedback_timer.arm_once().map_err(|_| RtError::Timer)?;

        // Wait for feedback
        let mut poll_fds = [libc::pollfd {
            fd: feedback_timer.fd(),
            events: libc::POLLIN,
            revents: 0,
        }];

        loop {
            let result =
                unsafe { libc::poll(poll_fds.as_mut_ptr(), poll_fds.len() as libc::nfds_t, -1) };
            if result < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(RtError::Poll);
            } else {
                break;
            }
        }

        if poll_fds[0].revents & libc::POLLIN != 0 {
            info!("FEEDBACK TIMER ELAPSED");
            timekeeper.on_feedback()
        }

        // with_timeout(self.wait_for_TPDO(), 250us); // How?

        // Bookkeeping
        let cycle_timing = timekeeper.end_cycle();

        info!("{:?} - SYNC", cycle_timing);

        Ok(())
    }

    fn process_can_rx(&self, can: &CanSocket) {
        for _ in 0..RT_CONFIG.can_frames_per_poll {
            match can.read_frame() {
                Ok(frame) => {
                    info!(
                        "CAN RX id={:#x} data={:?}",
                        frame.raw_id(),
                        &frame.data()[..frame.data().len()]
                    );
                }

                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
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

    fn process_cmd_rx(&mut self) {
        info!("process_cmd_rx");
        // Drain command queue into internal state transition queue
        match self.cmd_channel_rx.drain() {
            Ok(drain) => {
                trace!("Drain success");
                for cmd in drain {
                    if let Err(err) = self.cmd_queue.push(cmd) {
                        error!("RT CMD RX unable to push {:?} - {:?}", cmd, err);
                    }

                    info!("RT pushed cmd: {:?} -> {}", cmd, self.cmd_queue);
                }
            }
            Err(err) => {
                error!("Unable to drain command queue: {:?}", err);
            }
        }
    }

    fn transition_to(&mut self, cmd: RtCommand) {
        match cmd {
            RtCommand::Shutdown => self.state = RtState::Shutdown,
            RtCommand::Reconfigure => {
                // TODO: get new config somehow
                self.state = RtState::Reconfiguring
            }
            RtCommand::SingleCycle => self.state = RtState::SingleCycle,
            RtCommand::Cyclic => self.state = RtState::Cyclic,
        }
    }

    fn send_sync(&self, can: &CanSocket) {
        can.write_frame(&self.sync_frame)
            .expect("Unable to write SYNC");
    }

    fn feedback_timer_elapsed(&self) -> _ {
        todo!()
    }
}

use std::os::fd::{AsFd, AsRawFd};

use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Frame, Socket};
use tracing::{error, info, trace};

use crate::{
    consts::{MAX_CAN_RX_PER_POLL, RT_CYCLE_PERIOD},
    rt::{cmd::channel::CmdReceiver, timekeeper::TimeKeeper, timerfd::TimerFd},
};

const SYNC: usize = 0;
const CAN_RX: usize = 1;
const CMD_RX: usize = 2;

pub struct RtEngine<const N: usize> {
    can_interface: String,
    cmd_rx: CmdReceiver<N>,
}

impl<const N: usize> RtEngine<N> {
    pub fn start(can_interface: String, cmd_rx: CmdReceiver<N>) -> std::thread::JoinHandle<()> {
        let mut rt = Self {
            can_interface,
            cmd_rx,
        };
        std::thread::spawn(move || rt.run())
    }

    fn run(&mut self) {
        info!("RT Thread started");

        let mut timekeeper = TimeKeeper::new();

        let sync = CanFrame::from_raw_id(0x080, &[]).expect("failed to construct SYNC frame");

        let timer = TimerFd::from_period(RT_CYCLE_PERIOD).expect("Failed to create RT timer");

        let mut can = CanSocket::open(&self.can_interface).expect("Unable to open CAN interface");
        can.set_nonblocking(false)
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
        use crate::rt::cmd::RtCommand::*;

        info!("process_cmd_rx");
        // Drain command queue
        match self.cmd_rx.drain() {
            Ok(drain) => {
                info!("Drain success");
                for cmd in drain {
                    info!("RT received cmd: {:?}", cmd);
                }
            }
            Err(err) => {
                error!("Unable to drain command queue: {:?}", err);
            }
        }
    }
}

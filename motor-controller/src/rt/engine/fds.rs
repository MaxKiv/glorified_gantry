use libc::pollfd;

use crate::rt::timerfd::TimerFd;

const FEEDBACK_FD: usize = 0;
const CMD_RX_FD: usize = 1;
const CAN_RX_FD: usize = 2;
const SYNC_FD: usize = 3;

pub struct RtEngineFds {
    pub sync_timer: TimerFd,
    pub feedback_timer: TimerFd,

    poll_fds: [pollfd; 4],
}

impl RtEngineFds {
    pub fn new(
        sync_timer: TimerFd,
        feedback_timer: TimerFd,
        feedback_fd: pollfd,
        cmd_rx_fd: pollfd,
        can_rx_fd: pollfd,
        sync_fd: pollfd,
    ) -> Self {
        let poll_fds = [feedback_fd, cmd_rx_fd, can_rx_fd, sync_fd];

        Self {
            poll_fds,
            sync_timer,
            feedback_timer,
        }
    }

    pub fn feedback_deadline_elapsed(&self) -> bool {
        self.poll_fds[FEEDBACK_FD].revents & libc::POLLIN != 0
    }

    pub fn cmd_received(&self) -> bool {
        self.poll_fds[CMD_RX_FD].revents & libc::POLLIN != 0
    }

    pub fn can_frame_received(&self) -> bool {
        self.poll_fds[CAN_RX_FD].revents & libc::POLLIN != 0
    }

    pub fn sync_timer_elapsed(&self) -> bool {
        self.poll_fds[SYNC_FD].revents & libc::POLLIN != 0
    }

    pub fn poll(&mut self) -> i32 {
        unsafe {
            libc::poll(
                self.poll_fds.as_mut_ptr(),
                self.poll_fds.len() as libc::nfds_t,
                -1,
            )
        }
    }
}

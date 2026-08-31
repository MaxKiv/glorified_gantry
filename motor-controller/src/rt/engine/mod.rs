pub mod cfg;
pub mod cycle_rx;

use std::{
    mem::MaybeUninit,
    os::fd::{AsFd, AsRawFd},
    sync::atomic::{AtomicUsize, Ordering},
    thread::JoinHandle,
};

use libc::pollfd;
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Frame, Socket};
use tracing::{error, info, trace, warn};

use crate::{
    canopen::{MessageType, frame::CanOpenFrame, pdo::PdoType},
    consts::{
        MAX_NODE_ID, RT_CONFIG,
        pdo::gantry::{
            DEFAULT_ACTIVE_GANTRY_PDOCFG, DEFAULT_GANTRY_PDOCFG, HGantryActivePdoConfig,
            HGantryPdoConfig, TEST_MOTORS, get_initial_gantry_state,
        },
    },
    fifo::Fifo,
    rt::{
        MotorFeedback, RtError,
        cmd::{RtCommand, channel::CmdReceiver},
        engine::{
            cfg::{
                ConstRtEngineConfig, DEFAULT_MUT_RT_ENGINE_CFG, GantryMotor, MotorState,
                MutableRtEngineConfig, TEST_CONST_RT_ENGINE_CFG,
            },
            cycle_rx::{CyclePhase, CycleState},
        },
        timekeeper::TimeKeeper,
        timerfd::{TimerFd, TimerType},
    },
};

const SYNC_TIMER_FD: usize = 0;
const FEEDBACK_TIMER_FD: usize = 1;
const CAN_RX_FD: usize = 2;
const CMD_RX_FD: usize = 3;
const CMD_CHANNEL_SIZE: usize = RT_CONFIG.cmd_channel_size;
const CMD_QUEUE_SIZE: usize = RT_CONFIG.cmd_channel_size;

#[derive(Default, PartialEq)]
enum RtState {
    #[default]
    Idle,
    SingleCycle,
    Cyclic,
    Reconfiguring,
    Faulted,
    Shutdown,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MotorSetpoint {
    // TPDO
    pub control_word: u16,
    pub operation_mode: i8,
    pub target_position: i32,
    pub target_velocity: i32,
    pub target_torque: i16,
}

/// N = Number of Managed Motors
pub struct RtEngine {
    can_interface: String,
    can: CanSocket,
    cmd_channel_rx: CmdReceiver<CMD_CHANNEL_SIZE>,
    cmd_queue: Fifo<RtCommand, CMD_QUEUE_SIZE>,
    state: RtState,
    sync_frame: CanFrame,
    const_rt_cfg: ConstRtEngineConfig,
    managed_motors: [Option<GantryMotor>; MAX_NODE_ID],
    motor_state: [Option<MotorState>; MAX_NODE_ID],
    motor_feedback: [Option<MotorFeedback>; MAX_NODE_ID],
    motor_setpoint: [Option<MotorSetpoint>; MAX_NODE_ID],
    poll_fds: [pollfd; 4],
    sync_timer: TimerFd,
    feedback_timer: TimerFd,
    timekeeper: TimeKeeper,
    cycle_state: CycleState,
}

impl RtEngine {
    pub fn start(
        can_interface: String,
        cmd_rx: CmdReceiver<CMD_CHANNEL_SIZE>,
    ) -> JoinHandle<Result<(), RtError>> {
        let sync: CanFrame =
            CanFrame::from_raw_id(0x080, &[]).expect("failed to construct SYNC frame");

        let timekeeper = TimeKeeper::new();

        // Constructs a new sync cycle timer
        // implemented as absolute monotonic clock
        let sync_timer =
            TimerFd::from_period_monotonic(RT_CONFIG.cycle_period, TimerType::Absolute)
                .expect("Failed to create RT timer");

        // Construccts a new feedback timer
        // implemented as relative monotonic clock
        let feedback_timer =
            TimerFd::from_period_monotonic(RT_CONFIG.feedback_period, TimerType::Relative)
                .expect("Failed to create RT timer");

        let can = CanSocket::open(&can_interface).expect("Unable to open CAN interface");
        can.set_nonblocking(false)
            .expect("Unable to set CAN socket nonblocking");

        // Construct poll FDs
        let poll_fds = [
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
                fd: cmd_rx.fd(),
                events: libc::POLLIN,
                revents: 0,
            },
        ];

        let cycle_state = CycleState {
            cycle: 0u64,
            phase: CyclePhase::SendingSync,
            received: [false; 4],
        };

        let mut rt = Self {
            can_interface,
            cmd_channel_rx: cmd_rx,
            state: RtState::default(),
            cmd_queue: Fifo::<RtCommand, CMD_QUEUE_SIZE>::new(),
            sync_frame: sync,
            can,
            poll_fds,
            timekeeper,
            sync_timer,
            feedback_timer,
            const_rt_cfg: TEST_CONST_RT_ENGINE_CFG,
            cycle_state,
            managed_motors: TEST_MOTORS,
            motor_state: todo!(),
            motor_feedback: todo!(),
            motor_setpoint: todo!(),
        };

        // Spawn RT engine thread
        std::thread::spawn(move || rt.run())
    }

    fn run(&mut self) -> Result<(), RtError> {
        info!("RT Thread started");

        // Start SYNC timer
        self.sync_timer.arm().map_err(|_| RtError::Timer)?;

        // Main loop
        while self.state != RtState::Shutdown {
            // Wait for event to happen: Poll FDs
            if self.poll() < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                panic!("poll failed: {error}");
            }

            // Event happend: Service events
            if self.cmd_received() {
                self.process_cmd_rx();
            }
            if self.can_frame_received() {
                self.process_can_rx();
            }
            if self.cycle_state.all_sync_feedback_received() {
                self.sync_feedback_received();
            }
            if self.feedback_time_elapsed() {
                self.feedback_timer_elapsed();
            }
            if self.sync_timer_elapsed() {
                self.start_sync_cycle()?;
            }

            trace!("RT engine looping");
        }

        warn!("RT engine shutting down...");
        Ok(())
    }

    fn feedback_time_elapsed(&self) -> bool {
        self.poll_fds[FEEDBACK_TIMER_FD].revents & libc::POLLIN != 0
    }

    fn cmd_received(&self) -> bool {
        self.poll_fds[CMD_RX_FD].revents & libc::POLLIN != 0
    }

    fn can_frame_received(&self) -> bool {
        self.poll_fds[CAN_RX_FD].revents & libc::POLLIN != 0
    }

    fn sync_timer_elapsed(&self) -> bool {
        self.poll_fds[SYNC_TIMER_FD].revents & libc::POLLIN != 0
    }

    fn start_sync_cycle(&mut self) -> Result<(), RtError> {
        // Time cycle
        self.timekeeper.start_new_cycle();

        // Check timer expirations
        let expirations = self.sync_timer.expirations().expect("TimerFD read failed");

        // Check if we missed any
        if expirations != 1 {
            error!("RT overrun: {} SYNC timer expirations", expirations);
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
        self.send_sync(&self.can);

        // Setup feedback timer
        self.feedback_timer.arm_once().map_err(|_| RtError::Timer)?;
        self.timekeeper.start_feedback();

        // Bookkeeping
        self.cycle_state.transition_to(CyclePhase::WaitingForTpdos);

        Ok(())
    }

    fn process_can_rx(&mut self) {
        for _ in 0..RT_CONFIG.can_frames_per_poll {
            match self.can.read_frame() {
                Ok(frame) => {
                    info!(
                        "CAN RX id={:#x} data={:?}",
                        frame.raw_id(),
                        &frame.data()[..frame.data().len()]
                    );

                    let Ok(parsed) = CanOpenFrame::from_canframe(frame) else {
                        error!(
                            "Unable to parse CAN RX id={:#x} data={:?}",
                            frame.raw_id(),
                            &frame.data()
                        );
                        continue;
                    };
                    info!("CAN RX Parsed: {:?}", frame);

                    match parsed.msg {
                        MessageType::PDO(pdo) => {
                            // What type of PDO is this?
                            if pdo.pdo_type == PdoType::RPDO {
                                // Match rpdo msg node id to a managed motor
                                if let Some((idx, motor)) = self
                                    .managed_motors
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(i, m)| Some((i, m.as_ref()?)))
                                    .find(|(_, m)| m.node_id == pdo.node_id)
                                {
                                    // RPDO matched, parse into [`MotorFeedback`]
                                    let state = self.motor_state[idx].as_ref().unwrap();
                                    let feedback = self.motor_feedback[idx].as_mut().unwrap();

                                    match state.pdo_cfg.parse_rpdo(&pdo, feedback) {
                                        Ok(_) => {
                                            tracing::info!(
                                                "RPDO parsing & feedback update success! -
                                                    motor: {}",
                                                motor.node_id.get()
                                            )

                                            // TODO: update cycle state feedback processed for motor.node_id
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to parse RPDO: {}", e)
                                        }
                                    }
                                } else {
                                    tracing::error!(
                                        "RPDO parse error - unable to match {:?} to any managed motor",
                                        pdo
                                    );
                                }
                            }
                        }
                        _ => {
                            warn!("TODO: impl logic for this CAN RX {:?}", parsed);
                        }
                    }
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
            RtCommand::Shutdown => {
                self.state = RtState::Shutdown;
            }
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

    fn feedback_timer_elapsed(&mut self) {
        // Check timer expirations
        let expirations = self
            .feedback_timer
            .expirations()
            .expect("TimerFD read failed");

        // Check if we missed any
        if expirations != 1 {
            error!("RT overrun: {} feedback timer expirations", expirations);
        }

        self.timekeeper.end_feedback();
        let ct = self.timekeeper.end_cycle(self.cycle_state.cycle);
        error!("Device feedback did not arrive in time! - {:?}", ct);

        // TODO: what to do here?
        // Transition to ErrorState?
        // Accept a single feedback delayed cycle and restart?
        self.cycle_state.phase = CyclePhase::SendingSync; // TODO: remove
        self.timekeeper.end_cycle(self.cycle_state.cycle);
    }

    /// Triggers on sync feedback received
    /// Handles CyclePhase::SendingRpdoS & CyclePhase::SdoWindow
    fn sync_feedback_received(&mut self) {
        // TPDOs are in, bookkeeping
        self.timekeeper.end_feedback();
        self.cycle_state.transition_to(CyclePhase::SendingRpdoS);

        // CAN_RX should have parsed TPDO into coherent current [`MotorState`]

        //   if (x-axis skew > large) -> ESTOP

        // Snapshot RPDO Setpoints

        //   if (torque mode) {
        //      Calculate x axis skew compensation
        //      Add skew compensation to setpoint
        //   }

        // Transmit PDO

        // Construct CycleFeedback
        // Notify tokio of it somehow

        // Enter "SDO window" cycle phase
        self.cycle_state.transition_to(CyclePhase::SdoWindow);

        // Fetch pending SDO from Tokio somehow
        // Add those to a list, poll these in main polling loop as lowest priority and send
        // conditioned on cycle_state.phase = CyclePhase::SdoWindow

        self.cycle_state.transition_to(CyclePhase::SendingSync);
        let cycle_timing = self.timekeeper.end_cycle(self.cycle_state.cycle);
        info!("{:?} - SYNC", cycle_timing);
    }

    fn poll(&mut self) -> i32 {
        unsafe {
            libc::poll(
                self.poll_fds.as_mut_ptr(),
                self.poll_fds.len() as libc::nfds_t,
                -1,
            )
        }
    }
}

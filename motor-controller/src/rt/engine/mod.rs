pub mod cfg;
pub mod cycle_rx;

use std::{
    os::fd::{AsFd, AsRawFd},
    thread::JoinHandle,
};

use libc::pollfd;
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Frame, Socket};
use tracing::{error, info, trace, warn};

use crate::{
    canopen::{MessageType, frame::CanOpenFrame, pdo::PdoType},
    consts::{MAX_NODE_ID, RT_CONFIG, pdo::gantry::TEST_MOTORS},
    fifo::Fifo,
    rt::{
        MotorFeedback, RtError,
        cmd::{ReconfigurePayload, RtCommand, channel::CmdReceiver},
        engine::{
            cfg::{ConstRtEngineConfig, GantryMotor, MotorState, TEST_CONST_RT_ENGINE_CFG},
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

        let motor_state = [const { None }; MAX_NODE_ID];
        let motor_feedback = [const { None }; MAX_NODE_ID];
        let motor_setpoint = [const { None }; MAX_NODE_ID];
        let cycle_state = CycleState::new();

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
            motor_state,
            motor_feedback,
            motor_setpoint,
        };

        // Spawn RT engine thread
        std::thread::spawn(move || rt.run())
    }

    fn run(&mut self) -> Result<(), RtError> {
        info!("RT Thread started");

        // Arm SYNC timer
        self.sync_timer.arm().map_err(|_| RtError::Timer)?;

        // Main loop
        loop {
            // Error condition
            if self.state == RtState::Faulted {
                self.to_safe_state();
            }
            if self.state == RtState::Shutdown {
                break; // Exit from main rt loop
            }

            // Wait for event to happen: Poll FDs
            if self.poll() < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                error!("poll failed: {error}");
                self.to_error_state("poll failed");
            }

            // Event happend: Service events
            if self.can_frame_received() {
                self.process_can_rx();
            }

            if self.cmd_received() {
                self.process_cmd_rx();
            }

            // ALL states
            // TODO: check assumption - Motors produce TPDO (when transmission_type=on_sync) within
            // feedback window in every operation mode
            if self.cycle_state.is_all_cycle_feedback_received() {
                self.sync_feedback_received();
            } else if self.feedback_time_elapsed() {
                self.feedback_timer_elapsed();
            }

            if self.sync_timer_elapsed() {
                self.start_sync_cycle()?;
            }

            trace!("RT engine looping\n");
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
        self.timekeeper.on_sync_cycle_start();

        self.sync_timer.reset().map_err(|_| RtError::Timer)?;

        // Process cmds
        if !self.cmd_queue.is_empty() {
            let cmd = self
                .cmd_queue
                .pop()
                .expect("CMD Queue checks out to be non-empty, but pop() returns Error");
            match cmd {
                RtCommand::Shutdown => {
                    self.set_rt_state(RtState::Faulted);
                    // TODO: cyclephase?
                    return Ok(());
                }
                RtCommand::Reconfigure(new_cfg) => {
                    self.reconfigure_motor(new_cfg)?;
                    self.set_rt_state(RtState::Reconfiguring);
                    // TODO: get new config from somewhere?
                    // TODO: set mapping
                }
                RtCommand::SingleCycle => {
                    self.set_rt_state(RtState::SingleCycle);
                }
                RtCommand::Cyclic => {
                    self.set_rt_state(RtState::Cyclic);
                }
                RtCommand::Idle => {
                    self.set_rt_state(RtState::Idle);
                }
            }
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
                                if let Some((motor_idx, motor)) = self
                                    .managed_motors
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(i, m)| Some((i, m.as_ref()?)))
                                    .find(|(_, m)| m.node_id == pdo.node_id)
                                {
                                    // RPDO matched, parse into [`MotorFeedback`]
                                    let state = self.motor_state[motor_idx].as_ref().unwrap();
                                    let feedback = self.motor_feedback[motor_idx].as_mut().unwrap();

                                    // Was this RPDO num expected for this motor?
                                    if let Some(_) = state.pdo_cfg.rpdo[pdo.num] {
                                        // if self.cycle_state.pdo_state[motor_idx][pdo.num].expected {
                                        // Try to update motor feedback for this motor
                                        match state.pdo_cfg.parse_rpdo(&pdo, feedback) {
                                            Ok(_) => {
                                                info!(
                                                    "RPDO parsing & feedback update success for motor: {}",
                                                    motor.node_id.get()
                                                );

                                                // Update cycle state feedback processed for this motor
                                                self.cycle_state
                                                    .process_rpdo_received(&pdo, motor_idx);
                                            }
                                            Err(e) => {
                                                error!("Failed to parse RPDO: {}", e)
                                            }
                                        }
                                    } else {
                                        error!(
                                            "RPDO parse error - unable to match {:?} to any managed
                                            motor, ignoring...",
                                            pdo
                                        );
                                    }
                                } else {
                                    // this RPDO num was not expected for this motor
                                    warn!(
                                        "Node {} Unexpected RPDO {} Received!, ignoring",
                                        pdo.node_id.get(),
                                        pdo.num
                                    );
                                }
                            }
                        }
                        _ => {
                            error!("TODO: impl logic for this CAN RX {:?}", parsed);
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

        // TODO:
        // CAN_RX should have parsed TPDO into coherent current [`MotorState`]

        //   if (x-axis skew > large) -> ESTOP

        // Snapshot RPDO Setpoints
        self.snapshot_motor_setpoints();

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

    fn to_error_state(&mut self, arg: &'static str) {
        self.set_rt_state(RtState::Faulted);
        todo!("TODO error_state for: {}", arg)
    }

    fn to_safe_state(&mut self) {
        // TODO: Move motors to safe setpoint / state
        todo!("Move motors to safe setpoint / state -> E/QUICK STOP drives?");
        self.set_rt_state(RtState::Shutdown);
    }

    fn set_rt_state(&mut self, new_state: RtState) {
        self.state = new_state;
    }

    fn reconfigure_motor(&self, new_cfg: ReconfigurePayload) -> Result<(), RtError> {
        // is this a valid motor?
        let Some(motor) = self
            .managed_motors
            .iter()
            .filter_map(|x| Some(x.as_ref()?))
            .find(|m| m.node_id == new_cfg.motor)
        else {
            return Err(RtError::InvalidMotor);
        };

        // Valid motor: Reconfigure PDO mappping

        // TODO: Get list of default params for this given operationmode
        // Do all the sdo calls
        // steal from parametrise_motor
        can.write_frame(&self.sync_frame)
            .expect("Unable to write SYNC");

        Ok(())
    }
}

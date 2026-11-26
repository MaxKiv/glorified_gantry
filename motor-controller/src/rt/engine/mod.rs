pub mod cycle_rx;
pub mod fds;

use std::thread::JoinHandle;

use socketcan::{CanSocket, EmbeddedFrame, Frame, Socket};
use tracing::{debug, error, info, trace, warn};
use uom::si::{f64::Length, length::millimeter};

use crate::{
    canopen::{CanOpen, frame::CanOpenFrame},
    consts::RT_CONFIG,
    fifo::Fifo,
    frontend::{cmd::GantryCommand, event::GantryEvent},
    gantry::{Gantry, cfg::GantryCfg, setpoint::GantrySetpoint},
    motor::ctx::TickCtx,
    rt::{
        RtError,
        cmd::channel::CmdReceiver,
        engine::{
            cycle_rx::{CyclePhase, CycleState},
            fds::RtEngineFds,
        },
        event::EventSender,
        feedback::CycleFeedback,
        timekeeper::CycleTiming,
        timerfd::{TimerFd, TimerType},
    },
};

const CMD_CHANNEL_SIZE: usize = RT_CONFIG.cmd_channel_size;
const CMD_QUEUE_SIZE: usize = RT_CONFIG.cmd_channel_size;

#[derive(Debug, Default, PartialEq)]
pub enum RtState {
    #[default]
    Idle,
    SingleCycle,
    Cyclic,
    Reconfiguring,
    StartingUp,
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

pub struct RtEngine {
    can_interface: String,
    canopen: CanOpen,

    // Frontend Channels
    /// Command channel Frontend -> RtEngine
    cmd_channel_rx: CmdReceiver<CMD_CHANNEL_SIZE>,
    /// FIFO GantryCommand stack for later consumption
    cmd_queue: Fifo<GantryCommand, CMD_QUEUE_SIZE>,
    /// Event channel RtEngine -> Frontend
    event_tx: EventSender<GantryEvent>,

    state: RtState,

    // axi: [Option<GantryAxis>; Axis::COUNT],
    // sync_timer: TimerFd,
    // feedback_timer: TimerFd,
    // const_rt_cfg: ConstRtEngineConfig,
    cfg: GantryCfg,
    gantry: Gantry,

    current_cmd: GantryCommand,

    fds: RtEngineFds,
    cycle_state: CycleState,
}

impl RtEngine {
    pub fn start(
        can_interface: String,
        cmd_rx: CmdReceiver<CMD_CHANNEL_SIZE>,
        event_tx: EventSender<GantryEvent>,
        gantry_cfg: GantryCfg,
    ) -> JoinHandle<Result<(), RtError>> {
        let can = CanSocket::open(&can_interface).expect("Unable to open CAN interface");
        can.set_nonblocking(true)
            .expect("Unable to set CAN socket nonblocking");
        let canopen = CanOpen::new(can);

        // Constructs a new sync cycle timer
        // implemented as absolute monotonic clock
        let sync_timer =
            TimerFd::from_period_monotonic("Sync", RT_CONFIG.cycle_period, TimerType::Absolute)
                .expect("Failed to create RT timer");

        // Constructs a new feedback timer
        // implemented as relative monotonic clock
        let feedback_timer = TimerFd::from_period_monotonic(
            "Feedback",
            RT_CONFIG.feedback_period,
            TimerType::Relative,
        )
        .expect("Failed to create RT timer");

        // Construct poll FDs
        let sync_fd = sync_timer.fd();
        let feedback_fd = feedback_timer.fd();
        let can_fd = canopen.raw_fd();
        let cmd_fd = cmd_rx.fd();
        let fds = RtEngineFds::new(
            sync_timer,
            feedback_timer,
            libc::pollfd {
                fd: sync_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: feedback_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: can_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: cmd_fd,
                events: libc::POLLIN,
                revents: 0,
            },
        );

        // construct gantry
        let gantry = Gantry::with_cfg(gantry_cfg.clone());

        let mut rt = Self {
            can_interface,
            canopen,
            cmd_channel_rx: cmd_rx,
            state: RtState::default(),
            cmd_queue: Fifo::<GantryCommand, CMD_QUEUE_SIZE>::new(),
            cycle_state: CycleState::new(),
            current_cmd: GantryCommand::default(),
            event_tx,
            cfg: gantry_cfg,
            gantry,
            fds,
        };

        // Spawn RT engine thread
        std::thread::spawn(move || rt.run())
    }

    fn run(&mut self) -> Result<(), RtError> {
        info!("RT Thread started");

        // Arm SYNC timer
        self.fds.sync_timer.arm().map_err(|_| RtError::Timer)?;
        // Main RT loop
        loop {
            // NOTE: anything here triggers on every poll() resulution
            // So after CAN RX, Timer expirations, CMD RX, etc

            // Error condition
            debug!(
                system = "RtEngine",
                "RT loop start in state: {:?}", self.state
            );
            match self.state {
                RtState::StartingUp => {
                    // Gantry Starting up
                    // TODO:? Handle here or in cia402motor?
                }
                RtState::Reconfiguring => {
                    // Gantry Reconfiguring
                    // TODO:? Handle here or in cia402motor?
                }
                RtState::Faulted => {
                    self.to_safe_state();
                }
                RtState::Shutdown => {
                    break; // Exit from main rt loop
                }
                _ => {
                    // Intentionally empty
                }
            }

            // Schedule non-i/o work for all governed axi (and thus their motors)
            let mut ctx = TickCtx::new(&mut self.canopen);
            self.gantry.tick(&mut ctx);

            // Wait for event to happen: Poll FDs
            if self.fds.poll() < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                error!("poll failed: {error}");
                self.to_error_state("poll failed");
            }

            // Drain CAN
            if self.fds.can_frame_received() {
                self.process_can_rx();
            }
            // Drain Frontend CMDs
            if self.fds.cmd_received() {
                self.process_cmd_rx();
            }

            // Handle timing events.
            if self.cycle_state.phase == CyclePhase::WaitingForTpdo {
                if self.gantry.all_cycle_tpdo_received() {
                    self.sync_feedback_received();
                } else if self.fds.feedback_deadline_elapsed() {
                    self.feedback_deadline_exceeded();
                }
            }

            if self.fds.sync_timer_elapsed() {
                self.start_sync_cycle()?;
            }

            // Service SDO
            // This sends at most 1 SDO call out to a device, in an attempt to guarantee never
            // exceeding the synchronisation window
            if self.state == RtState::Cyclic {
                // Only send out SDO messages during the SDO Window in cyclic modes
                if self.cycle_state.phase == CyclePhase::SdoWindow {
                    // TODO: check against time left in SDO window?
                    self.canopen.send_single_sdo();
                }
            } else {
                // In other modes we can send out SDO messages whenever
                self.canopen.send_single_sdo();
            }

            trace!("RT engine looping\n");
        }

        warn!("RT engine shutting down...");
        Ok(())
    }

    fn process_can_rx(&mut self) {
        for _ in 0..RT_CONFIG.can_frames_per_poll {
            // Read raw can frame
            match self.canopen.read_raw_frame() {
                Ok(frame) => {
                    info!(
                        system = "CAN",
                        "CAN RX id={:#x} data={:?}",
                        frame.raw_id(),
                        &frame.data()[..frame.data().len()]
                    );

                    // Try parse into CANOpen data frame
                    let Ok(parsed) = CanOpenFrame::from_canframe(frame) else {
                        error!(
                            system = "CAN",
                            "Unable to parse CAN RX id={:#x} data={:?}",
                            frame.raw_id(),
                            &frame.data()
                        );
                        continue;
                    };
                    info!(system = "CAN", "CAN RX Parsed: {:?}", frame);

                    // Dispatch adressed messages to their recipient
                    if let Some(msg_id) = parsed.node_id {
                        for axis in self.gantry.get_axi_mut() {
                            if axis
                                .managed_node_ids()
                                .find(|node| *node == msg_id)
                                .is_some()
                            {
                                axis.process_canopen_msg(parsed);
                                break;
                            }
                        }
                    } else {
                        // Ignore broadcast CANOpen frame
                    }
                }

                Err(error) => {
                    // Ignore WouldBlock
                    if error.kind() != std::io::ErrorKind::WouldBlock {
                        error!("CAN RX error: {error}");
                    }
                }
            }
        }
    }

    fn process_cmd_rx(&mut self) {
        info!("process_cmd_rx");
        // Drain command queue into internal state transition queue
        // the commands in the internal queue will be drained and and acted upon at the start of a
        // SYNC cycle
        match self.cmd_channel_rx.drain() {
            Ok(drain) => {
                trace!("Drain success");
                for cmd in drain {
                    if let Err(err) = self.cmd_queue.push(cmd.clone()) {
                        error!("RT CMD RX unable to push {:?}", err);
                    }

                    info!("RT pushed cmd: {:?} -> {}", cmd, self.cmd_queue);
                }
            }
            Err(err) => {
                error!("Unable to drain command queue: {:?}", err);
            }
        }
    }

    fn start_sync_cycle(&mut self) -> Result<(), RtError> {
        // Reset SYNC timer
        self.fds.sync_timer.reset().map_err(|_| RtError::Timer)?;

        // Calculate timing metrics for previous cycle
        self.cycle_state
            .transition_cycle_phase(CyclePhase::CycleEnd);

        // Start new SYNC cycle
        self.cycle_state
            .transition_cycle_phase(CyclePhase::SendingSync);

        // Process cmds, drains internal command queue
        // TODO
        if !self.cmd_queue.is_empty() {
            let cmd = self
                .cmd_queue
                .pop()
                .expect("CMD Queue checks out to be non-empty, but pop() returns Error");

            match cmd.clone() {
                GantryCommand::CyclicSetpoint(gantry_setpoint)
                | GantryCommand::Setpoint(gantry_setpoint) => {
                    self.gantry.new_gantry_setpoint(gantry_setpoint);
                }
                GantryCommand::Home => {
                    self.gantry
                        .new_gantry_setpoint(GantrySetpoint::new_home_all_axis());
                    self.set_rt_state(RtState::SingleCycle);
                }
                GantryCommand::Idle => {
                    // TODO: Transition to cia402 disabled?
                    self.set_rt_state(RtState::Idle);
                }
            }
            self.current_cmd = cmd;
        }

        // Write SYNC
        self.canopen.send_sync().map_err(|e| RtError::CanOpen(e))?;

        // Setup feedback timer
        self.fds
            .feedback_timer
            .arm_once()
            .map_err(|_| RtError::Timer)?;

        // Bookkeeping
        self.cycle_state
            .transition_cycle_phase(CyclePhase::WaitingForTpdo);

        // Control flow continues in on_sync_feedback(), or feedback_deadline_exceeded(), depending
        // on whether feedback actually arrived

        Ok(())
    }

    fn feedback_deadline_exceeded(&mut self) {
        // Check timer expirations
        let expirations = self
            .fds
            .feedback_timer
            .expirations()
            .expect("TimerFD read failed");

        // Check if we missed any
        if expirations != 1 {
            error!("RT overrun: {} feedback timer expirations", expirations);
        }

        let cycle_timing = self.cycle_state.get_cycle_timing();
        error!(
            "Device feedback did not arrive in time! - {:?}",
            cycle_timing
        );

        // TODO: what to do here?
        self.cycle_state.on_feedback_deadline_elapsed();
    }

    fn sync_feedback_received(&mut self) {
        debug!(system = "RtEngine", "sync_feedback_received",);

        // TPDOs are in, bookkeeping
        self.cycle_state
            .transition_cycle_phase(CyclePhase::ReceivedTpdo);

        // Calculate axis skew
        let skew = self.gantry.get_axi_skew();
        debug!(system = "RtEngine", "Axis skew: {}", skew);

        // ESTOP if axis skew is too large
        if skew.exceeds(Length::new::<millimeter>(RT_CONFIG.max_axis_skew_mm)) {
            error!(
                system = "RtEngine",
                "Excessive axis skew detected: {} > {:?}mm", skew, RT_CONFIG.max_axis_skew_mm,
            );

            self.gantry.do_e_stop();
            self.cycle_state
                .transition_cycle_phase(CyclePhase::SdoWindow);
        }

        for axis in self.gantry.get_axi_mut() {
            axis.on_sync_feedback();
        }

        // Collect RPDO for each motor in the gantry
        debug!(system = "RtEngine", "Collect & Transmit RPDO",);
        for axis in self.gantry.get_axi_ref() {
            let axis_rpdos = axis.get_rpdos();

            // Send each RPDO over can,
            // NOTE: regarding timing, this means setpoints arrive sequentially
            // at the motors within 1-5ms delay.
            // In profile modes motors act immediately on new setpoints
            // In cyclic modes the motors wait for the next SYNC
            self.canopen
                .send_motor_rpdoset(axis_rpdos.master, &axis.master.id);
            if let Some(ref slave) = axis.slave {
                assert!(
                    axis_rpdos.slave.is_some(),
                    "RtEngine - axis.get_rpdos() returns None for slave axis, but axis.slave is_some()"
                );
                self.canopen
                    .send_motor_rpdoset(axis_rpdos.slave.unwrap(), &slave.id);
            }
        }

        self.cycle_state
            .transition_cycle_phase(CyclePhase::SentRpdo);

        // Construct CycleFeedback for async consumption
        let gantry_feedback = self.gantry.get_gantry_feedback();
        let cycle_timing = self.cycle_state.get_cycle_timing();
        let cycle_feedback = CycleFeedback::new(cycle_timing, skew, gantry_feedback);
        self.event_tx.push(GantryEvent::Feedback(cycle_feedback));
        // Notify async side
        self.event_tx.notify();

        // Enter "SDO window" cycle phase
        self.cycle_state
            .transition_cycle_phase(CyclePhase::SdoWindow);

        // The SDO window is further picked up in the main RT loop
    }

    fn to_error_state(&mut self, arg: &'static str) {
        self.set_rt_state(RtState::Faulted);
        todo!("TODO error_state for: {}", arg)
    }

    fn to_safe_state(&mut self) {
        self.do_e_stop();

        self.set_rt_state(RtState::Shutdown);
    }

    fn set_rt_state(&mut self, new_state: RtState) {
        self.state = new_state;
    }

    fn do_e_stop(&mut self) {
        self.gantry.do_e_stop();
        self.state = RtState::Faulted;
    }
}

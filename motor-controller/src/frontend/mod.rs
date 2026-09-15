use tokio::runtime::Handle;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use uom::si::f64::Length;
use uom::si::f64::Torque;
use uom::si::f64::Velocity;

use crate::consts::RT_CONFIG;
use crate::oms::home::HomingSetpoint;
use crate::oms::setpoint::Setpoint;
use crate::rt::cmd::channel::CmdSender;

const CMD_CHANNEL_SIZE: usize = RT_CONFIG.cmd_channel_size;

/// Non-RT part
/// Responsible for
///  configuration             
///  external commands        
///  logging/UI/RPC          

#[derive(Debug, Clone)]
pub struct PositionCommand {
    target: Length,
    vel: Velocity,
    relative: bool,
}

#[derive(Debug, Clone)]
pub struct VelocityCommand {
    target: Velocity,
}

#[derive(Debug, Clone)]
pub struct TorqueCommand {
    target: Torque,
}

#[derive(Debug, Clone)]
pub struct GantrySetpoint {
    pub x: Option<Setpoint>,
    pub y: Option<Setpoint>,
    pub z: Option<Setpoint>,
}

impl GantrySetpoint {
    pub fn new_home_all_axis() -> Self {
        GantrySetpoint {
            x: Some(Setpoint::Home(HomingSetpoint::default())),
            y: Some(Setpoint::Home(HomingSetpoint::default())),
            z: Some(Setpoint::Home(HomingSetpoint::default())),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum GantryCommand {
    /// Motors are Responsible for interpolation
    Setpoint(GantrySetpoint),
    /// You are Responsible for interpolation
    CyclicSetpoint(GantrySetpoint),
    /// Home motors
    Home,
    /// De Energise motors
    #[default]
    Idle,
    // ResetFaults, ?
}

#[derive(Debug, thiserror::Error)]
pub enum GantryFrontendError {
    #[error("Command Channel Closed")]
    CommandChannelClosed,
}

struct GantrySnapshot {
    sequence: u64,

    state: GantryState,

    axes: [AxisSnapshot; 3],

    drives: [DriveSnapshot; 6],

    faults: FaultSummary,
}
struct AxisSnapshot {
    position: Position,
    velocity: Velocity,
    torque: Torque,

    skew: Option<Position>,
}
struct DriveSnapshot {
    nmt_state: NmtState,
    cia402_state: Cia402State,

    position: Position,
    velocity: Velocity,
    torque: Torque,

    following_error: Position,
}

// | Thing                      | Owner                                         |
// | -------------------------- | --------------------------------------------- |
// | CAN socket                 | RT                                            |
// | PDO RX/TX                  | RT                                            |
// | SYNC                       | RT                                            |
// | SDO transactions           | non-RT, executed through RT-owned CAN service |   ! <-- I question the need for this, RT does parametrisation?
// | NMT state                  | RT                                            |
// | CiA-402 state              | RT                                            |
// | controlword generation     | RT                                            |
// | statusword interpretation  | RT                                            |
// | Profile Position handshake | RT                                            |
// | actual position/velocity   | RT                                            |
// | trajectory execution       | RT                                            |
// | axis synchronization       | RT                                            |
// | machine safety state       | RT                                            |
// | configuration              | non-RT                                        |
// | external commands          | non-RT                                        |
// | logging/UI/RPC             | non-RT                                        |
/// Non-RT part of [`MotionController`]
pub struct GantryFrontend {
    cmd_rx: mpsc::Receiver<GantryCommand>,
    event_tx: broadcast::Sender<GantryEvent>,
}

impl GantryFrontend {
    pub fn start_on(runtime: &Handle, rt_cmd_tx: CmdSender<CMD_CHANNEL_SIZE>) -> GantryHandle {
        pub const CMD_CH_SIZE: usize = 64;
        pub const EVENT_CH_SIZE: usize = 64;

        let (cmd_tx, cmd_rx) = mpsc::channel(CMD_CH_SIZE);
        let (event_tx, event_rx) = broadcast::channel(EVENT_CH_SIZE);

        let controller = GantryFrontend { cmd_rx, event_tx };

        let joinset = JoinSet::new();
        joinset.spawn_on(GantryFrontend::bridge_commands(cmd_rx, rt_cmd_tx), runtime);
        joinset.spawn_on(
            GantryFrontend::bridge_events(event_tx, rt_event_rx),
            runtime,
        );

        GantryHandle {
            cmd_tx,
            event_rx,
            joinset,
        }
    }

    /// Bridges the [`GantryCommand`] send to the async [`GantryFrontend`] to the RT [`GantryController`]
    async fn bridge_commands(
        mut cmd_rx: mpsc::Receiver<GantryCommand>,
        rt_cmd_tx: CmdSender<CMD_CHANNEL_SIZE>,
    ) -> Result<(), GantryFrontendError> {
        loop {
            // Wait for command to arrive at async mpsc channel
            let cmd = cmd_rx
                .recv()
                .await
                .ok_or(GantryFrontendError::CommandChannelClosed)?;

            // Send over Frontend -> RT SPSC channel
            rt_cmd_tx.send(&cmd);
        }
    }

    /// Bridges the [`GantryCommand`] send to the async [`GantryFrontend`] to the RT [`GantryController`]
    async fn bridge_events(
        mut event_tx: broadcast::Sender<GantryEvent>,
        rt_event_rx: todo!(),
    ) -> Result<(), GantryFrontendError> {
        loop {
            let event = rt_event_rx.recv();

            if let Err(err) = event_tx.send(event) {
                tracing::error!("Unable to bridge GantryEvent: {:?}", err);
            }
        }
    }
}

pub struct GantryHandle {
    cmd_tx: mpsc::Sender<GantryCommand>,
    event_rx: broadcast::Receiver<GantryEvent>,
    joinset: JoinSet<Result<(), GantryFrontendError>>,
}

impl GantryHandle {
    pub async fn join(self) -> Vec<Result<(), GantryFrontendError>> {
        self.joinset.join_all().await
    }

    pub async fn send(
        &self,
        cmd: GantryCommand,
    ) -> Result<(), mpsc::error::SendError<GantryCommand>> {
        self.cmd_tx.send(cmd).await
    }
}

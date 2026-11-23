pub mod cmd;
pub mod event;

use crate::frontend::cmd::GantryCommand;
use crate::rt::event::EventReceiver;
use tokio::runtime::Handle;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use uom::si::f64::Length;
use uom::si::f64::Torque;
use uom::si::f64::Velocity;

use crate::canopen::nmt::NmtState;
use crate::cia402::Cia402State;
use crate::consts::RT_CONFIG;
use crate::frontend::event::GantryEvent;
use crate::rt::cmd::channel::CmdSender;
use crate::rt::engine::RtState;

pub const CMD_CH_SIZE: usize = RT_CONFIG.cmd_channel_size;
pub const EVENT_CH_SIZE: usize = RT_CONFIG.event_channel_size;

/// Non-RT part
/// Responsible for
///  configuration             
///  external commands        
///  logging/UI/RPC          
#[derive(Debug, thiserror::Error)]
pub enum GantryFrontendError {
    #[error("Command Channel Closed")]
    CommandChannelClosed,
    #[error("Event Channel Closed")]
    EventChannelClosed,
}

struct GantrySnapshot {
    sequence: u64,
    state: RtState,
    axes: [AxisSnapshot; 3],
    drives: [DriveSnapshot; 6],
    // TODO:
    // fault: Option<GantryError>,
}

struct AxisSnapshot {
    position: Length,
    velocity: Velocity,
    torque: Torque,
    skew: Option<Length>,
}

struct DriveSnapshot {
    nmt_state: NmtState,
    cia402_state: Cia402State,

    position: Length,
    velocity: Velocity,
    torque: Torque,

    following_error: Length,
}

// | Thing                      | Owner        |
// | -------------------------- | ------------ |
// | CAN socket                 | RT           |
// | PDO RX/TX                  | RT           |
// | SYNC                       | RT           |
// | SDO transactions           | RT           |
// | NMT state                  | RT           |
// | CiA-402 state              | RT           |
// | controlword generation     | RT           |
// | statusword interpretation  | RT           |
// | Profile Position handshake | RT           |
// | actual position/velocity   | RT           |
// | trajectory execution       | RT           |
// | axis synchronization       | RT           |
// | machine safety state       | RT           |
// | external commands          | non-RT       |
// | logging/UI/RPC             | non-RT       |
/// Non-RT part of [`MotionController`]
pub struct GantryFrontend {
    cmd_rx: mpsc::Receiver<GantryCommand>,
    event_tx: broadcast::Sender<GantryEvent>,
}

impl GantryFrontend {
    pub fn start_on(
        runtime: &Handle,
        rt_cmd_tx: CmdSender<CMD_CH_SIZE>,
        rt_event_rx: EventReceiver<GantryEvent>,
    ) -> GantryHandle {
        let (cmd_tx, cmd_rx) = mpsc::channel(CMD_CH_SIZE);
        let (event_tx, event_rx) = broadcast::channel(EVENT_CH_SIZE);

        let mut task_joinset = JoinSet::new();
        task_joinset.spawn_on(GantryFrontend::bridge_commands(cmd_rx, rt_cmd_tx), runtime);
        task_joinset.spawn_on(
            GantryFrontend::bridge_events(event_tx, rt_event_rx),
            runtime,
        );

        GantryHandle {
            cmd_tx,
            event_rx,
            joinset: task_joinset,
        }
    }

    /// Bridges the [`GantryCommand`] send to the async [`GantryFrontend`] to the RT [`GantryController`]
    async fn bridge_commands(
        mut cmd_rx: mpsc::Receiver<GantryCommand>,
        rt_cmd_tx: CmdSender<CMD_CH_SIZE>,
    ) -> Result<(), GantryFrontendError> {
        loop {
            // Wait for command to arrive at async mpsc channel
            let cmd = cmd_rx
                .recv()
                .await
                .ok_or(GantryFrontendError::CommandChannelClosed)?;

            // Send over Frontend -> RT SPSC channel
            rt_cmd_tx
                .send(&cmd)
                .expect("Unable to send frontend cmd -> RtEngine");
        }
    }

    /// Bridges the [`GantryEvent`] send to the async [`GantryFrontend`] to the RT [`RtEngine`]
    async fn bridge_events(
        event_tx: broadcast::Sender<GantryEvent>,
        rt_event_rx: EventReceiver<GantryEvent>,
    ) -> Result<(), GantryFrontendError> {
        let mut rx = rt_event_rx.into_async().unwrap();

        loop {
            rx.readable()
                .await
                .map_err(|_| GantryFrontendError::EventChannelClosed)?;

            while let Some(event) = rx.try_recv() {
                // Errors here means nobody is listening; silently discard
                let _ = event_tx.send(event);
            }

            let num_dropped = rx.take_dropped();
            if num_dropped > 0 {
                tracing::warn!("RT event ring overflowed, dropped {} events", num_dropped);
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

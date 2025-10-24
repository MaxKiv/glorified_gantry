use oze_canopen::interface::CanOpenInterface;
use tokio::{sync::broadcast, time::Instant};
use tracing::*;

use crate::{
    axis::{AxisConfig, AxisMotors, receiver::AxisEventReceiver},
    command::{
        GantryCommand,
        handler::{CommandHandle, CommandHandler},
    },
    event::{
        GantryEvent,
        handler::{FeedbackHandle, FeedbackHandler},
    },
    setpoint::translator::{SetpointTranslator, scaling::DeviceScaling},
    sync::{SyncMaster, SyncMasterHandle},
};

pub struct Gantry {
    sync: SyncMasterHandle,
    cmd_handler: CommandHandle,
    feedback_handler: FeedbackHandle,
}

impl Gantry {
    pub async fn start(
        canopen: CanOpenInterface,
        x_cfg: Option<AxisConfig>,
        y_cfg: Option<AxisConfig>,
        z_cfg: Option<AxisConfig>,
    ) -> anyhow::Result<Self> {
        info!("Starting SYNC Master");
        let sync = SyncMaster::init(canopen.clone());

        info!("Starting X Axis");
        let (x_motors, x_recv) = if let Some(cfg) = x_cfg {
            let (motors, recv) =
                Gantry::start_axis(cfg, canopen.clone(), sync.get_sync_receiver()).await?;
            (Some(motors), Some(recv))
        } else {
            (None, None)
        };

        info!("Starting Y Ayis");
        let (y_motors, y_recv) = if let Some(cfg) = y_cfg {
            let (motors, recv) =
                Gantry::start_axis(cfg, canopen.clone(), sync.get_sync_receiver()).await?;
            (Some(motors), Some(recv))
        } else {
            (None, None)
        };

        info!("Starting Z Azis");
        let (z_motors, z_recv) = if let Some(cfg) = z_cfg {
            let (motors, recv) =
                Gantry::start_axis(cfg, canopen.clone(), sync.get_sync_receiver()).await?;
            (Some(motors), Some(recv))
        } else {
            (None, None)
        };

        info!("Initialising Setpoint Translator");
        let translator = SetpointTranslator::new(DeviceScaling::default());

        info!("Starting Feedback Handler");

        let feedback_handler = FeedbackHandler::init(x_recv, y_recv, z_recv, translator.clone());

        info!("Starting Command Handler");
        let cmd_handler = CommandHandler::init(x_motors, y_motors, z_motors, translator.clone());

        info!("Gantry Initialized!");
        Ok(Self {
            sync,
            cmd_handler,
            feedback_handler,
        })
    }

    /// Sends a [`GantryCommand`] to this Gantry
    pub async fn send_command(
        &self,
        cmd: GantryCommand,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<GantryCommand>> {
        self.cmd_handler.cmd_tx.send(cmd).await
    }

    /// Start all motors connected to single gantry axis, returning a handle to them and their
    /// event receivers
    async fn start_axis(
        cfg: AxisConfig,
        canopen: CanOpenInterface,
        sync_rx: broadcast::Receiver<Instant>,
    ) -> anyhow::Result<(AxisMotors, AxisEventReceiver)> {
        let axis = cfg.axis.clone();

        // Construct this axis's motor drivers
        let motor = AxisMotors::new(canopen.clone(), cfg, sync_rx).await?;

        // Construct the motor event receiver for this axis
        let recv = AxisEventReceiver::new(
            axis,
            motor.master.event_rx.resubscribe(),
            motor
                .slave
                .as_ref()
                .map(|slave| slave.event_rx.resubscribe()),
        );

        Ok((motor, recv))
    }

    pub fn get_event_rx(&self) -> broadcast::Receiver<GantryEvent> {
        self.feedback_handler.gantry_rx.resubscribe()
    }
}

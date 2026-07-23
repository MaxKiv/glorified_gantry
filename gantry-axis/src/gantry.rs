use oze_canopen::interface::CanOpenInterface;
use tokio::{
    sync::{broadcast, mpsc},
    time::Instant,
};
use tracing::*;

use crate::{
    axis::{AxisConfig, AxisMotors, receiver::AxisEventReceiver},
    cfg::GantryConfig,
    command::{
        GantryCommand,
        handler::{CommandHandle, CommandHandler},
    },
    event::{
        GantryMotorEvent,
        handler::{FeedbackHandle, FeedbackHandler},
    },
    setpoint::translator::SetpointTranslator,
    sync::{SyncMaster, SyncMasterHandle},
};

pub struct Gantry {
    canopen: CanOpenInterface,
    sync: SyncMasterHandle,
    cmd_handler: CommandHandle,
    feedback_handler: FeedbackHandle,
    pub cfg: GantryConfig,
}

impl Gantry {
    pub async fn start(canopen: CanOpenInterface, cfg: GantryConfig) -> anyhow::Result<Self> {
        info!("Starting SYNC Master");
        let sync = SyncMaster::init(canopen.clone());

        info!("Starting X Axis");
        // Initialize X Axis motors and return their handles + device scaling
        // All of this is a NOOP if this axis is disabled by setting its cfg to None
        let (x_motors, x_recv, x_translator) = if let Some(ref cfg) = cfg.x {
            let translator = SetpointTranslator::new(&cfg.scaling);

            let (motors, recv) =
                Gantry::start_axis(cfg.clone(), canopen.clone(), sync.get_sync_receiver()).await?;
            (Some(motors), Some(recv), Some(translator))
        } else {
            (None, None, None)
        };

        info!("Starting Y Ayis");
        // Initialize X Axis motors and return their handles + device scaling
        // All of this is a NOOP if this axis is disabled by setting its cfg to None
        let (y_motors, y_recv, y_translator) = if let Some(ref cfg) = cfg.y {
            let translator = SetpointTranslator::new(&cfg.scaling);

            let (motors, recv) =
                Gantry::start_axis(cfg.clone(), canopen.clone(), sync.get_sync_receiver()).await?;
            (Some(motors), Some(recv), Some(translator))
        } else {
            (None, None, None)
        };

        info!("Starting Z Azis");
        // Initialize X Axis motors and return their handles + device scaling
        // All of this is a NOOP if this axis is disabled by setting its cfg to None
        let (z_motors, z_recv, z_translator) = if let Some(ref cfg) = cfg.z {
            let translator = SetpointTranslator::new(&cfg.scaling);

            let (motors, recv) =
                Gantry::start_axis(cfg.clone(), canopen.clone(), sync.get_sync_receiver()).await?;
            (Some(motors), Some(recv), Some(translator))
        } else {
            (None, None, None)
        };

        info!("Starting Feedback Handler");
        let feedback_handler = FeedbackHandler::init(
            x_recv,
            y_recv,
            z_recv,
            x_translator.clone(),
            y_translator.clone(),
            z_translator.clone(),
        );

        info!("Starting Command Handler");
        let cmd_handler = CommandHandler::init(
            x_motors,
            y_motors,
            z_motors,
            x_translator,
            y_translator,
            z_translator,
        );

        info!("Gantry Initialized!");
        Ok(Self {
            canopen,
            sync,
            cmd_handler,
            feedback_handler,
            cfg,
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
        let master_id = cfg.master.clone();
        let slave_id = cfg.slave.clone();

        // Construct this axis's motor drivers
        let motor = AxisMotors::new(canopen.clone(), cfg, sync_rx).await?;

        // Construct the motor event receiver for this axis
        let recv = AxisEventReceiver::new(
            axis,
            master_id,
            motor.master.event_rx.resubscribe(),
            slave_id,
            motor
                .slave
                .as_ref()
                .map(|slave| slave.event_rx.resubscribe()),
        );

        Ok((motor, recv))
    }

    pub fn get_event_rx(&self) -> broadcast::Receiver<GantryMotorEvent> {
        self.feedback_handler.gantry_rx.resubscribe()
    }

    pub fn get_cmd_tx(&self) -> mpsc::Sender<GantryCommand> {
        self.cmd_handler.cmd_tx.clone()
    }
}

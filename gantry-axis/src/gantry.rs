use tracing::*;

use crate::{
    axis::AxisMotors,
    command::handler::{CommandHandler, CommandHandlerHandle},
    default::*,
    sync::{SyncMaster, SyncMasterHandle},
};

pub struct Gantry {
    sync: SyncMasterHandle,
    cmd_handler: CommandHandlerHandle,
}

impl Gantry {
    pub async fn default() -> anyhow::Result<Self> {
        info!("Starting CAN interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        info!("Starting SYNC Master");
        let sync = SyncMaster::init(canopen.clone());

        info!("Starting X Axis motor drivers");
        let x_axis =
            AxisMotors::new(canopen.clone(), DEFAULT_X_CONFIG, sync.get_sync_receiver()).await?;
        info!("Starting Y Axis motor drivers");
        let y_axis =
            AxisMotors::new(canopen.clone(), DEFAULT_Y_CONFIG, sync.get_sync_receiver()).await?;
        info!("Starting Z Axis motor drivers");
        let z_axis =
            AxisMotors::new(canopen.clone(), DEFAULT_Z_CONFIG, sync.get_sync_receiver()).await?;

        info!("Starting Command Handler");
        let cmd_handler = CommandHandler::init(x_axis, y_axis, z_axis);

        Ok(Self { sync, cmd_handler })
    }
}

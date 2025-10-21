pub mod setpoint;

use gantry_cia402::driver::{Cia402Driver, builder::Cia402DriverBuilder};
use oze_canopen::interface::CanOpenInterface;
use tokio::{sync::broadcast, time::Instant};
use tracing::info;

#[derive(Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

pub struct AxisConfig {
    pub axis: Axis,
    pub master: u8,
    pub slave: Option<u8>,
}

pub struct AxisMotors {
    axis: Axis,
    master: Cia402Driver,
    slave: Option<Cia402Driver>,
}

impl AxisMotors {
    pub async fn new(
        canopen: CanOpenInterface,
        axis_config: AxisConfig,
        sync_rx: broadcast::Receiver<Instant>,
    ) -> anyhow::Result<Self> {
        let master_node = axis_config.master;
        let slave_node = axis_config.slave;
        let axis = axis_config.axis;
        info!(
            "Initializing {axis:?} motors - master id: {master_node} slave id: {:?}",
            slave_node
        );

        info!("Constructing {axis:?} master driver at id {master_node}",);
        let master = Cia402DriverBuilder::new(master_node)
            .with_canopen(canopen.clone())
            .with_default_pdo_mappings()
            .with_default_parameters()
            .with_sync_receiver(sync_rx.resubscribe())
            .build()
            .await?;

        let slave = if let Some(slave_id) = slave_node {
            info!("Constructing {axis:?} slave driver at id {slave_id}",);
            Some(
                Cia402DriverBuilder::new(slave_id)
                    .with_canopen(canopen.clone())
                    .with_default_pdo_mappings()
                    .with_default_parameters()
                    .with_sync_receiver(sync_rx.resubscribe())
                    .build()
                    .await?,
            )
        } else {
            info!("No slave found for axis {axis:?}",);
            None
        };

        Ok(Self {
            axis,
            master,
            slave,
        })
    }
}

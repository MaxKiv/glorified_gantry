pub mod receiver;
pub mod setpoint;

use gantry_cia402::{
    comms::sdo::SdoAction,
    driver::{Cia402Driver, builder::Cia402DriverBuilder, command::MotorCommand},
};
use oze_canopen::interface::CanOpenInterface;
use tokio::{sync::broadcast, time::Instant};
use tracing::*;

use crate::setpoint::translator::scaling::DeviceScaling;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// Configuration struct for a single gantry axis
#[derive(Clone)]
pub struct AxisConfig {
    /// What axis is this config for
    pub axis: Axis,
    /// Whats the masters CANopen node id
    pub master: u8,
    /// The slave's node id, if there is one
    pub slave: Option<u8>,
    /// Required parameters for each motor of this axis
    pub params: &'static [SdoAction<'static>],
    /// Define how to map from SI units <-> Motor units
    pub scaling: DeviceScaling,
}

pub struct AxisMotors {
    pub axis: Axis,
    pub master: Cia402Driver,
    pub slave: Option<Cia402Driver>,
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
        let params = axis_config.params;
        info!(
            "Initializing {axis:?} motors - master id: {master_node} slave id: {:?}",
            slave_node
        );

        info!("Constructing {axis:?} master driver at id {master_node}",);
        let master = Cia402DriverBuilder::new(master_node)
            .with_canopen(canopen.clone())
            .with_default_pdo_mappings()
            .with_parameters(params)
            .with_sync_receiver(sync_rx.resubscribe())
            .build()
            .await?;

        let slave = if let Some(slave_id) = slave_node {
            info!("Constructing {axis:?} slave driver at id {slave_id}",);
            Some(
                Cia402DriverBuilder::new(slave_id)
                    .with_canopen(canopen.clone())
                    .with_default_pdo_mappings()
                    .with_parameters(params)
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

    /// Send given motorcommand to the master and slave motors of this axis
    pub fn send_command_to_motors(&self, command: &MotorCommand) {
        let master_cmd = command.clone();
        let slave_cmd = command.clone();

        info!("Axis {:?} sending command: {command:?}", self.axis);

        // Send to master driver
        if let Err(e) = self.master.cmd_tx.send(master_cmd) {
            error!(
                "Axis {:?} unable to send command to Master: {command:?} - {e}",
                self.axis
            );
        }

        // Send to slave driver if that exists
        if let Some(slave) = &self.slave
            && let Err(e) = slave.cmd_tx.send(slave_cmd)
        {
            error!(
                "Axis {:?} unable to send command to Slave: {command:?} - {e}",
                self.axis
            );
        }
    }
}

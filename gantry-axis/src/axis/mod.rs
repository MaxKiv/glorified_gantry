pub mod receiver;
pub mod setpoint;

use gantry_cia402::{
    comms::sdo::SdoAction,
    driver::{
        AxisMaster, AxisSlave, Cia402Driver, builder::Cia402DriverBuilder, command::MotorCommand,
        identifier::Cia402Identifier,
    },
};
use oze_canopen::interface::CanOpenInterface;
use tokio::{sync::broadcast, time::Instant};
use tracing::*;

use crate::setpoint::translator::scaling::DeviceScaling;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub master: Cia402Identifier,
    /// The slave's node id, if there is one
    pub slave: Option<Cia402Identifier>,
    /// Required parameters for each motor of this axis
    pub params: &'static [SdoAction<'static>],
    /// Define how to map from SI units <-> Motor units
    pub scaling: DeviceScaling,
}

pub struct AxisMotors {
    pub axis: Axis,
    pub master: Cia402Driver<AxisMaster>,
    pub slave: Option<Cia402Driver<AxisSlave>>,
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
            .as_master()
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
                    .as_slave_with_master(&master)
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
        warn!("xxx Axis {:?} sending command: {command:?}", self.axis);

        // Send command to all Cia402Drivers that make up this axis
        if let Err(e) = self.master.cmd_tx.send(command.clone()) {
            error!(
                "Axis {:?} unable to send command to Master: {command:?} - {e}",
                self.axis
            );
        }
    }
}

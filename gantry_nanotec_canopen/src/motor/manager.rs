use std::time::Duration;

use anyhow::Result;
use oze_canopen::{
    canopen::JoinHandles,
    interface::CanOpenInterface,
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
    sync,
};
use tracing::*;

use crate::{error::MotorError, motor::nanotec_motor::NanotecMotor, pdo::mapping::PdoMapping};

const DEFAULT_BAUDRATE: u32 = 1_000_000;
const DEFAULT_SYNC_PERIOD: Duration = Duration::from_millis(10);

pub struct MotorManager {
    canopen_interface: CanOpenInterface,
    canopen_handles: JoinHandles,
}

impl MotorManager {
    pub fn new(can_interface: String, bitrate: Option<u32>) -> Self {
        let bitrate = bitrate.unwrap_or(DEFAULT_BAUDRATE);
        trace!("Constructing new MotorManager using {can_interface} @ {bitrate}");

        trace!("Set up tasks that manage CAN Tx/Rx traffic & auto-reconnect logic");
        let (canopen_interface, canopen_handles) =
            oze_canopen::canopen::start(can_interface, Some(bitrate));

        trace!("Set up SYNC server with period {DEFAULT_SYNC_PERIOD:?}");
        let sync_server = sync::Server::start(canopen_interface.clone());
        sync_server.set_period(Some(DEFAULT_SYNC_PERIOD));

        Self {
            canopen_interface,
            canopen_handles,
        }
    }

    pub async fn construct_motor(&self, node_id: u8) -> Result<NanotecMotor> {
        trace!("constructing motor with node id {node_id}");

        trace!("Setting up sdo client for node id {node_id}");
        let sdo = self
            .canopen_interface
            .get_sdo_client(node_id)
            .ok_or(MotorError::SdoClientConstruction)?;

        trace!("Motor boots into NMT::PreOperational -> set motor to NMT::Operational");
        self.canopen_interface
            .send_nmt(NmtCommand::new(
                NmtCommandSpecifier::StartRemoteNode,
                node_id,
            ))
            .await
            .map_err(MotorError::CanOpen)?;

        trace!("Construct motor handle for motor {node_id}");
        let motor = NanotecMotor::with_node_id(node_id, sdo, self);

        trace!("Parametrize motor {node_id}");
        motor.parametrize().await?;

        trace!("Configure PDO for motor {node_id}");
        motor
            .configure_pdo_mappings(PdoMapping::CUSTOM_RPDOS)
            .await?;
        motor
            .configure_pdo_mappings(PdoMapping::CUSTOM_TPDOS)
            .await?;

        Ok(motor)
    }
}

use std::time::Duration;

use anyhow::Result;
use oze_canopen::{
    canopen::JoinHandles,
    interface::CanOpenInterface,
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
    sync,
};

use crate::{error::MotorError, motor::nanotec_motor::NanotecMotor};

const DEFAULT_BAUDRATE: u32 = 1_000_000;

pub struct MotorManager {
    canopen_interface: CanOpenInterface,
    canopen_handles: JoinHandles,
}

impl MotorManager {
    pub fn new(can_interface: String, bitrate: Option<u32>) -> Self {
        // Set up tasks that manage CAN Tx/Rx traffic & auto-reconnect logic
        let (canopen_interface, canopen_handles) =
            oze_canopen::canopen::start(can_interface, bitrate.or(Some(DEFAULT_BAUDRATE)));

        // Set up the CANopen SYNC
        let sync_server = sync::Server::start(canopen_interface.clone());
        sync_server.set_period(Some(Duration::from_millis(1)));

        Self {
            canopen_interface,
            canopen_handles,
        }
    }

    pub async fn construct_motor(&self, node_id: u8) -> Result<NanotecMotor> {
        // Set up SDO client for this node id
        let sdo = self
            .canopen_interface
            .get_sdo_client(node_id)
            .ok_or(MotorError::SdoClientConstruction)?;

        // Motor boots into NMT::PreOperational -> Set motor to NMT::Operational
        self.canopen_interface
            .send_nmt(NmtCommand::new(
                NmtCommandSpecifier::StartRemoteNode,
                node_id,
            ))
            .await
            .map_err(MotorError::CanOpen)?;

        // Construct motor handle
        let motor = NanotecMotor::with_node_id(node_id, sdo, self);

        // Parametrize the motor
        motor.parametrize().await?;

        // Set up PDO for this motor
        motor.configure_pdo().await?;

        Ok(motor)
    }
}

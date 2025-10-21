pub mod common;

use std::time::Duration;

use tokio::task::{self};
use tracing::*;

#[cfg(test)]
mod tests {

    use gantry_cia402::{
        comms::pdo::mapping::{
            custom::{CUSTOM_PDOS, CUSTOM_TPDOS},
            minimal::MINIMAL_CYCLIC_SYNCHRONOUS_PDO_SET,
        },
        driver::{
            Cia402Driver, builder::Cia402DriverBuilder, command::MotorCommand, event::MotorEvent,
            receiver::subscriber::wait_for_event, state::Cia402State,
        },
        error::DriveError,
    };

    use crate::common::{NODE_ID, PARAMS, TIMEOUT, start_sync_master};

    use super::*;

    #[tokio::test]
    /// Test basic cia402 state transitions using PDO
    async fn test_cia402_pdo() -> Result<(), DriveError> {
        gantry_demo::setup_tracing();

        let node_id = NODE_ID;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        let sync_rx = start_sync_master(canopen.clone());

        info!("Initializing Cia402Driver for motor driver at node id {node_id}");
        let drive = Cia402DriverBuilder::new(node_id)
            .with_canopen(canopen.clone())
            .with_default_pdo_mappings()
            .with_parameters(PARAMS)
            .with_sync_receiver(sync_rx)
            .build()
            .await?;

        info!("Sending Command Disable");
        drive
            .cmd_tx
            .send(MotorCommand::Disable)
            .map_err(DriveError::CommandError)?;

        info!("Wait for Cia402State::ReadyToSwitchOn");
        wait_for_event(
            drive.event_rx.resubscribe(),
            MotorEvent::Cia402StateUpdate(Cia402State::ReadyToSwitchOn),
            TIMEOUT,
        )
        .await?;

        info!("Sending Command Enable");
        drive
            .cmd_tx
            .send(MotorCommand::Enable)
            .map_err(DriveError::CommandError)?;

        info!("Wait for Cia402State::OperationEnabled");
        wait_for_event(
            drive.event_rx.resubscribe(),
            MotorEvent::Cia402StateUpdate(Cia402State::OperationEnabled),
            TIMEOUT,
        )
        .await?;

        Ok(())
    }
}

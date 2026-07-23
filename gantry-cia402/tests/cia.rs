pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    use gantry_axis::sync::SyncMaster;
    use gantry_cia402::{
        driver::{
            builder::Cia402DriverBuilder, command::MotorCommand, event::MotorEvent,
            receiver::subscriber::wait_for_event, state::Cia402State,
        },
        error::DriveError,
    };

    use crate::common::{COMMS_TIMEOUT, PARAMS, TEST_MOTOR};

    use super::*;

    #[tokio::test]
    /// Test basic cia402 state transitions using PDO
    async fn test_cia402_pdo() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        let identifier = TEST_MOTOR;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        let sync_master = SyncMaster::init(canopen.clone());
        let sync_rx = sync_master.get_sync_receiver();

        info!("Initializing Cia402Driver for motor {identifier}");
        let drive = Cia402DriverBuilder::new(identifier)
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
            COMMS_TIMEOUT,
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
            COMMS_TIMEOUT,
        )
        .await?;

        Ok(())
    }
}

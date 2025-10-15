pub mod common;

use std::time::Duration;

use tokio::task::{self};
use tracing::*;

#[cfg(test)]
mod tests {

    use gantry_cia402::{
        comms::pdo::mapping::custom::CUSTOM_TPDOS,
        driver::{
            Cia402Driver, command::MotorCommand, event::MotorEvent,
            receiver::subscriber::wait_for_event, state::Cia402State,
        },
        error::DriveError,
        log::log_events,
    };

    use crate::common::{NODE_ID, PARAMS, RPDOS, TIMEOUT, TPDOS, start_feedback_task};

    use super::*;

    #[tokio::test]
    async fn test_homing() -> Result<(), DriveError> {
        gantry_demo::setup_tracing();

        let node_id = NODE_ID;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        info!("Initializing Cia402Driver for motor driver at node id {node_id}");
        let drive = Cia402Driver::init(node_id, canopen.clone(), PARAMS, RPDOS, TPDOS).await?;

        info!("Sending Command Disable");
        drive
            .cmd_tx
            .send(MotorCommand::Disable)
            .map_err(DriveError::CommandError)?;

        // info!("Wait for Cia402State::SwitchOnDisabled");
        // wait_for_event(
        //     drive.event_rx.resubscribe(),
        //     MotorEvent::Cia402StateUpdate(Cia402State::SwitchOnDisabled),
        //     TIMEOUT,
        // )
        // .await?;

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

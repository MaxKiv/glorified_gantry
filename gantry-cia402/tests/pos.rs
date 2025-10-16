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
    /// Test basic cia402 state transitions
    async fn test_cia402() -> Result<(), DriveError> {
        gantry_demo::setup_tracing();

        let node_id = NODE_ID;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        info!("Initializing Cia402Driver for motor driver at node id {node_id}");
        let drive = Cia402Driver::init(node_id, canopen.clone(), PARAMS, RPDOS, TPDOS).await?;

        info!("Sending Command Disable");
        drive
            .cmd_tx
            .send(MotorCommand::Cia402TransitionTo {
                target_state: Cia402State::SwitchOnDisabled,
            })
            .map_err(DriveError::CommandError)?;

        info!("Wait for Cia402State::ReadyToSwitchOn");
        wait_for_event(
            drive.event_rx.resubscribe(),
            MotorEvent::Cia402StateUpdate(Cia402State::SwitchOnDisabled),
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

        info!("Sending Home command");
        drive
            .cmd_tx
            .send(MotorCommand::Home)
            .map_err(DriveError::CommandError)?;

        info!("Wait for Homing completed event");
        wait_for_event(
            drive.event_rx.resubscribe(),
            MotorEvent::HomingFeedback {
                at_home: true,
                homing_completed: true,
                homing_error: false,
            },
            TIMEOUT,
        )
        .await?;

        info!("Doing position movement");
        drive
            .cmd_tx
            .send(MotorCommand::MoveRelative {
                delta: -3200,
                profile_velocity: 0x000001F4, // Default
            })
            .map_err(DriveError::CommandError)?;

        info!("Wait for PositionModeFeedback - target_reached");
        wait_for_event(
            drive.event_rx.resubscribe(),
            MotorEvent::PositionModeFeedback {
                target_reached: true,
                limit_exceeded: false,
                setpoint_acknowlegde: true,
                following_error: false,
            },
            TIMEOUT,
        )
        .await?;

        Ok(())
    }
}

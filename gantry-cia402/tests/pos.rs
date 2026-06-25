pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    const TEST_POSITION: i32 = 100;
    // const TEST_POSITION: i32 = 100;
    const TEST_SPEED: u32 = 10;

    use gantry_axis::{event::util::HOME_TIMEOUT, sync::SyncMaster};
    use gantry_cia402::{
        comms::sdo::SdoAction,
        driver::{
            Cia402Driver,
            builder::Cia402DriverBuilder,
            command::MotorCommand,
            event::MotorEvent,
            receiver::subscriber::{
                wait_for_event, wait_for_homing_completed, wait_for_setpoint_acknowledge,
                wait_for_target_reached,
            },
            startup::params::{TEST_PARAMS, default::DEMO_PARAMS},
            state::Cia402State,
        },
        error::DriveError,
    };
    use tokio::signal;

    use crate::common::{COMMS_TIMEOUT, PARAMS, POS_TIMEOUT};

    use super::*;

    #[tokio::test]
    /// Test basic cia402 state transitions
    async fn test_position_mode() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        pub const PARAMS: &[SdoAction] = TEST_PARAMS;

        let identifier = common::TEST_MOTOR;
        let node_id = identifier.node_id;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        let sync_master = SyncMaster::init(canopen.clone());
        let sync_rx = sync_master.get_sync_receiver();

        info!("Initializing Cia402Driver for motor driver at node id {node_id}");
        let drive = Cia402DriverBuilder::new(identifier)
            .with_canopen(canopen.clone())
            .with_default_pdo_mappings()
            .with_parameters(PARAMS)
            .with_sync_receiver(sync_rx)
            .build()
            .await?;

        // Create a task for the test logic
        let test_task = tokio::spawn(pos_test(drive));

        // Wait for either Ctrl-C or test completion
        tokio::select! {
            res = test_task => {
                res??;
            }
            _ = signal::ctrl_c() => {
                info!("Ctrl-C received — aborting test");
            }
        }

        Ok(())
    }

    async fn pos_test(drive: Cia402Driver) -> Result<(), DriveError> {
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

        info!("Sending Home command");
        drive
            .cmd_tx
            .send(MotorCommand::Home)
            .map_err(DriveError::CommandError)?;

        info!("Wait for Homing completed event");
        wait_for_homing_completed(drive.event_rx.resubscribe(), HOME_TIMEOUT).await?;

        for num in 1..=10 {
            info!("Doing absolute position movement forward # {num}");
            drive
                .cmd_tx
                .send(MotorCommand::MoveAbsolute {
                    target: TEST_POSITION,
                    profile_velocity: TEST_SPEED,
                })
                .map_err(DriveError::CommandError)?;

            info!("Wait for setpoint acknowledged event");
            wait_for_setpoint_acknowledge(drive.event_rx.resubscribe(), COMMS_TIMEOUT).await?;

            info!("Wait for target reached event");
            wait_for_target_reached(drive.event_rx.resubscribe(), COMMS_TIMEOUT, TEST_POSITION)
                .await?;

            info!("Doing position relative position movement backward # {num}");
            drive
                .cmd_tx
                .send(MotorCommand::MoveRelative {
                    delta: -TEST_POSITION,
                    profile_velocity: TEST_SPEED,
                })
                .map_err(DriveError::CommandError)?;

            info!("Wait for setpoint acknowledged event");
            wait_for_setpoint_acknowledge(drive.event_rx.resubscribe(), COMMS_TIMEOUT).await?;

            info!("Wait for target reached event");
            wait_for_target_reached(drive.event_rx.resubscribe(), COMMS_TIMEOUT, 0).await?;
        }

        Ok(())
    }
}

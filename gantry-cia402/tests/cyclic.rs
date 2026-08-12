pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    const TEST_SPEED: u32 = 100;
    const TEST_TORQUE: i16 = 69;
    const TEST_TIMEOUT: Duration = Duration::from_millis(1_000);

    use std::time::Duration;

    use gantry_axis::sync::{DEFAULT_SYNC_PERIOD, SyncMaster};
    use gantry_cia402::{
        driver::{
            Cia402Driver,
            builder::Cia402DriverBuilder,
            command::MotorCommand,
            cyclic::CyclicSynchronousMode,
            event::MotorEvent,
            receiver::subscriber::{wait_for_event, wait_for_target_reached},
            state::Cia402State,
        },
        error::DriveError,
    };
    use tokio::signal;

    use crate::common::{COMMS_TIMEOUT, PARAMS, TEST_MOTOR};

    use super::*;

    #[tokio::test]
    async fn test_cyclic_synchronous_position() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        let identifier = TEST_MOTOR;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        let sync_master = SyncMaster::init(canopen.clone());
        sync_master.set_sync_period(DEFAULT_SYNC_PERIOD)?;
        let sync_rx = sync_master.get_sync_receiver();

        info!("Initializing Cia402Driver for motor {identifier}");
        let drive = Cia402DriverBuilder::new(identifier)
            .with_canopen(canopen.clone())
            .with_default_pdo_mappings()
            .with_parameters(PARAMS)
            .with_sync_receiver(sync_rx, DEFAULT_SYNC_PERIOD)
            .build()
            .await?;

        // Create a task for the test logic
        let test_task = tokio::spawn(cyclic_synchronous_pos_test(drive));

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

    async fn cyclic_synchronous_pos_test(drive: Cia402Driver) -> Result<(), DriveError> {
        info!("Transitioning to ReadyToSwitchOn");
        drive
            .cmd_tx
            .send(MotorCommand::Cia402TransitionTo {
                target_state: Cia402State::ReadyToSwitchOn,
            })
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
            COMMS_TIMEOUT,
        )
        .await?;

        info!("Starting Cyclic Synchronous Torque test");
        drive
            .cmd_tx
            .send(MotorCommand::EnterCyclicSynchronousMode {
                mode: CyclicSynchronousMode::Position,
            })
            .map_err(DriveError::CommandError)?;

        info!("Wait for Homing completed event");
        wait_for_event(
            drive.event_rx.resubscribe(),
            MotorEvent::OperationModeUpdate(
                gantry_cia402::driver::oms::OperationMode::CyclicSynchronousPosition,
            ),
            COMMS_TIMEOUT,
        )
        .await?;

        for num in 1..=100 {
            let test_position = 20;
            info!("#{num} Setting {test_position}");
            drive
                .cmd_tx
                .send(MotorCommand::CyclicSynchronousPosition {
                    abs_target: test_position,
                })
                .map_err(DriveError::CommandError)?;

            info!("#{num} Wait for target reached");
            wait_for_target_reached(drive.event_rx.resubscribe(), COMMS_TIMEOUT, test_position)
                .await?;

            let test_position = 10;
            info!("#{num} Setting {test_position}");
            drive
                .cmd_tx
                .send(MotorCommand::CyclicSynchronousTorque { target: 0 })
                .map_err(DriveError::CommandError)?;

            info!("#{num} Wait for target reached");
            wait_for_target_reached(drive.event_rx.resubscribe(), COMMS_TIMEOUT, test_position)
                .await?;
        }

        Ok(())
    }
}

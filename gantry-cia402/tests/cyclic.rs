pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    const TEST_POSITIONS: Vec<i32> = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
        38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    ];

    const TEST_SPEED: u32 = 100;
    const TEST_TORQUE: i16 = 69;
    const TORQUE_TIMEOUT: Duration = Duration::from_millis(10_000);

    use std::time::Duration;

    use gantry_cia402::{
        driver::{
            Cia402Driver, builder::Cia402DriverBuilder, command::MotorCommand, event::MotorEvent,
            receiver::subscriber::wait_for_event, state::Cia402State,
        },
        error::DriveError,
    };
    use tokio::signal;

    use crate::common::{CYCLIC_PDOS, NODE_ID, PARAMS, RPDOS, TIMEOUT, TPDOS, start_sync_master};

    use super::*;

    #[tokio::test]
    async fn test_cyclic_synchronous_position() -> anyhow::Result<()> {
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

        for num in 1..=10 {
            info!("#{num} Setting {TEST_TORQUE} torque target");
            drive
                .cmd_tx
                .send(MotorCommand::CyclicSynchronousPosition {
                    abs_target: (),
                    target_velocity: (),
                    target_torque: (),
                })
                .map_err(DriveError::CommandError)?;

            info!("#{num} Wait for Torque Setpoint Reached event");
            wait_for_event(
                drive.event_rx.resubscribe(),
                MotorEvent::TorqueModeFeedback {
                    axis_braked: false,
                    setpoint_reached: true,
                    limit_exceeded: false,
                },
                TORQUE_TIMEOUT,
            )
            .await?;

            info!("#{num} Setting 0 torque");
            drive
                .cmd_tx
                .send(MotorCommand::SetTorque { target_torque: 0 })
                .map_err(DriveError::CommandError)?;

            info!("#{num} Wait for Torque Setpoint Reached event");
            wait_for_event(
                drive.event_rx.resubscribe(),
                MotorEvent::TorqueModeFeedback {
                    axis_braked: false,
                    setpoint_reached: true,
                    limit_exceeded: false,
                },
                TORQUE_TIMEOUT,
            )
            .await?;

            info!("#{num} Setting -{TEST_TORQUE} torque target");
            drive
                .cmd_tx
                .send(MotorCommand::SetTorque {
                    target_torque: -TEST_TORQUE,
                })
                .map_err(DriveError::CommandError)?;

            info!("#{num} Wait for Torque Setpoint Reached event");
            wait_for_event(
                drive.event_rx.resubscribe(),
                MotorEvent::TorqueModeFeedback {
                    axis_braked: false,
                    setpoint_reached: true,
                    limit_exceeded: false,
                },
                TORQUE_TIMEOUT,
            )
            .await?;

            info!("#{num} Setting 0 torque");
            drive
                .cmd_tx
                .send(MotorCommand::SetTorque { target_torque: 0 })
                .map_err(DriveError::CommandError)?;

            info!("#{num} Wait for Torque Setpoint Reached event");
            wait_for_event(
                drive.event_rx.resubscribe(),
                MotorEvent::TorqueModeFeedback {
                    axis_braked: false,
                    setpoint_reached: true,
                    limit_exceeded: false,
                },
                TORQUE_TIMEOUT,
            )
            .await?;
        }

        Ok(())
    }
}

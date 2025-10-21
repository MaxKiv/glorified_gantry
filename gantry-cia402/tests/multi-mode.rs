pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    const TEST_POSITION: i32 = -50;
    const TEST_SPEED: u32 = 100;
    const TEST_TORQUE: i16 = 69;
    const TORQUE_TIMEOUT: Duration = Duration::from_millis(10_000);

    use std::time::Duration;

    use gantry_cia402::{
        comms::pdo::mapping::{custom::CUSTOM_PDOS, minimal::MINIMAL_CYCLIC_SYNCHRONOUS_PDO_SET},
        driver::{
            Cia402Driver, builder::Cia402DriverBuilder, command::MotorCommand, event::MotorEvent,
            receiver::subscriber::wait_for_event, state::Cia402State,
        },
        error::DriveError,
    };
    use tokio::signal;

    use crate::common::{NODE_ID, PARAMS, TIMEOUT, start_sync_master};

    use super::*;

    #[tokio::test]
    /// Test basic cia402 state transitions
    async fn test_torque() -> anyhow::Result<()> {
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
        let test_task = tokio::spawn(torque_test_logic(drive));

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

    async fn torque_test_logic(drive: Cia402Driver) -> Result<(), DriveError> {
        info!("Sending Command Disable");
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

        for outer in 1..=300 {
            for num in 1..=3 {
                info!("#{num} Setting {TEST_TORQUE} torque target");
                drive
                    .cmd_tx
                    .send(MotorCommand::SetTorque {
                        target_torque: TEST_TORQUE,
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
                        target_torque: -(TEST_TORQUE),
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

            for num in 1..=3 {
                info!("Doing absolute position movement forward # {num}");
                drive
                    .cmd_tx
                    .send(MotorCommand::MoveAbsolute {
                        target: TEST_POSITION,
                        profile_velocity: TEST_SPEED,
                    })
                    .map_err(DriveError::CommandError)?;

                info!("Wait for PositionModeFeedback - setpoint_acknowlegded");
                wait_for_event(
                    drive.event_rx.resubscribe(),
                    MotorEvent::PositionModeFeedback {
                        target_reached: false,
                        limit_exceeded: false,
                        setpoint_acknowlegded: true,
                        following_error: false,
                    },
                    TIMEOUT,
                )
                .await?;
                wait_for_event(
                    drive.event_rx.resubscribe(),
                    MotorEvent::PositionModeFeedback {
                        target_reached: true,
                        limit_exceeded: false,
                        setpoint_acknowlegded: false,
                        following_error: false,
                    },
                    TIMEOUT,
                )
                .await?;

                info!("Doing position relative position movement backward # {num}");
                drive
                    .cmd_tx
                    .send(MotorCommand::MoveAbsolute {
                        target: -TEST_POSITION,
                        profile_velocity: TEST_SPEED,
                    })
                    .map_err(DriveError::CommandError)?;

                info!("Wait for PositionModeFeedback - target_reached");
                wait_for_event(
                    drive.event_rx.resubscribe(),
                    MotorEvent::PositionModeFeedback {
                        target_reached: false,
                        limit_exceeded: false,
                        setpoint_acknowlegded: true,
                        following_error: false,
                    },
                    TIMEOUT,
                )
                .await?;
                wait_for_event(
                    drive.event_rx.resubscribe(),
                    MotorEvent::PositionModeFeedback {
                        target_reached: true,
                        limit_exceeded: false,
                        setpoint_acknowlegded: false,
                        following_error: false,
                    },
                    TIMEOUT,
                )
                .await?;
            }
        }
        Ok(())
    }
}

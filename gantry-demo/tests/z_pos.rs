pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    use gantry_axis::{
        axis::{
            Axis,
            setpoint::{AxisSetpoint, PositionSetpoint},
        },
        command::GantryCommand,
        event::util::wait_for_target_reached,
        gantry::Gantry,
    };
    use tokio::signal;
    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::meter_per_second,
    };

    use gantry_demo::config::*;

    use crate::common::TIMEOUT;

    use super::*;

    #[tokio::test]
    /// Test basic cia402 state transitions
    async fn homing_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let gantry = Gantry::start(canopen, TEST_X_CONFIG, TEST_Y_CONFIG, TEST_Z_CONFIG).await?;

        // Create a task for the test logic
        let test_task = tokio::spawn(test_gantry_homing(gantry));

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

    async fn test_gantry_homing(gantry: Gantry) -> anyhow::Result<()> {
        info!("TEST: Homing gantry");

        gantry.send_command(GantryCommand::Home).await?;

        info!("TEST: wait on gantry homed");
        wait_for_target_reached(
            gantry.get_event_rx(),
            gantry_axis::event::util::TargetQuantity::Home(true),
            Axis::Z,
            TIMEOUT,
        )
        .await?;
        info!("TEST: Gantry homed!");

        let pos_target_x = Length::new::<millimeter>(60.0);
        let pos_target_y = Length::new::<millimeter>(30.0);
        let pos_target_z = Length::new::<millimeter>(1000.0);
        let vel = Velocity::new::<meter_per_second>(0.01);

        for _num in 1..10 {
            let setpoint = GantryCommand::Setpoint {
                x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: pos_target_x,
                    velocity: vel,
                })),
                y: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: pos_target_y,
                    velocity: vel,
                })),
                z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: pos_target_z,
                    velocity: vel,
                })),
            };
            info!("TEST: Sending setpoint: {:?}", setpoint.clone());

            gantry.send_command(setpoint.clone()).await?;

            wait_for_target_reached(
                gantry.get_event_rx(),
                gantry_axis::event::util::TargetQuantity::Position(
                    pos_target_z.get::<millimeter>(),
                ),
                Axis::Z,
                TIMEOUT,
            )
            .await;

            info!("TEST: setpoint: {:?} REACHED", setpoint.clone());

            let setpoint = GantryCommand::Setpoint {
                x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: -pos_target_x,
                    velocity: vel,
                })),
                y: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: -pos_target_y,
                    velocity: vel,
                })),
                z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: -pos_target_z,
                    velocity: vel,
                })),
            };
            info!("TEST: Sending setpoint: {:?}", setpoint.clone());

            gantry.send_command(setpoint.clone()).await?;

            wait_for_target_reached(
                gantry.get_event_rx(),
                gantry_axis::event::util::TargetQuantity::Position(
                    pos_target_z.get::<millimeter>(),
                ),
                Axis::Z,
                TIMEOUT,
            )
            .await;

            info!("TEST: setpoint: {:?} REACHED", setpoint.clone());
        }

        Ok(())
    }
}

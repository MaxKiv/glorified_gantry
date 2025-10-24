pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use gantry_axis::{
        axis::{
            Axis,
            setpoint::{AxisSetpoint, PositionSetpoint},
        },
        command::GantryCommand,
        event::util::wait_for_target_reached,
        gantry::Gantry,
    };
    use tokio::{signal, time::sleep};

    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::meter_per_second,
    };

    use gantry_demo::config::{TEST_X_CONFIG, TEST_Y_CONFIG, TEST_Z_CONFIG};

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
        gantry.send_command(GantryCommand::Home).await?;

        wait_for_target_reached(
            gantry.get_event_rx(),
            gantry_axis::event::util::TargetQuantity::Home(true),
            Axis::X,
            TIMEOUT,
        )
        .await?;

        tokio::time::sleep(Duration::from_secs(5)).await;

        let pos_target_x = Length::new::<millimeter>(30.0);
        let pos_target_z = Length::new::<millimeter>(10.0);
        let vel = Velocity::new::<meter_per_second>(0.001);

        let setpoint = GantryCommand::Setpoint {
            x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: -pos_target_x,
                velocity: vel,
            })),
            y: None,
            z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: -pos_target_z,
                velocity: vel,
            })),
        };
        gantry.send_command(setpoint).await?;

        tokio::try_join!(
            wait_for_target_reached(
                gantry.get_event_rx(),
                gantry_axis::event::util::TargetQuantity::Position(
                    pos_target_z.get::<millimeter>()
                ),
                Axis::Z,
                TIMEOUT,
            ),
            wait_for_target_reached(
                gantry.get_event_rx(),
                gantry_axis::event::util::TargetQuantity::Position(
                    pos_target_x.get::<millimeter>()
                ),
                Axis::X,
                TIMEOUT,
            ),
        )?;

        Ok(())
    }
}

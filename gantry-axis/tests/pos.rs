pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use gantry_axis::{
        axis::setpoint::{AxisSetpoint, PositionSetpoint},
        command::GantryCommand,
        gantry::Gantry,
    };
    use tokio::{signal, time::sleep};
    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::meter_per_second,
    };

    use crate::common::{TEST_X_CONFIG, TEST_Y_CONFIG, TEST_Z_CONFIG};

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

        sleep(Duration::from_secs(2)).await;

        let setpoint = GantryCommand::Setpoint {
            x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: Length::new::<millimeter>(10.0),
                velocity: Velocity::new::<meter_per_second>(0.01),
            })),
            y: None,
            z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: Length::new::<millimeter>(10.0),
                velocity: Velocity::new::<meter_per_second>(0.01),
            })),
        };

        gantry.send_command(setpoint).await?;

        sleep(Duration::from_secs(5)).await;

        let setpoint = GantryCommand::Setpoint {
            x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: Length::new::<millimeter>(-40.0),
                velocity: Velocity::new::<meter_per_second>(0.01),
            })),
            y: None,
            z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: Length::new::<millimeter>(-20.0),
                velocity: Velocity::new::<meter_per_second>(0.01),
            })),
        };

        gantry.send_command(setpoint).await?;

        sleep(Duration::from_secs(5)).await;

        Ok(())
    }
}

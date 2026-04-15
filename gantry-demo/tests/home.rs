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
        event::{
            GantryEvent,
            util::{
                wait_for_position_target_reached, wait_for_target_reached,
                wait_until_event_matches, wait_until_gantry_command_completed,
            },
        },
        gantry::Gantry,
    };
    use std::time::Duration;
    use tokio::{signal, time::sleep};
    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::meter_per_second,
    };

    use gantry_demo::config::*;

    use crate::common::{HOME_TIMEOUT, TIMEOUT};
    const TEST_VEL: f64 = 0.0001;

    use super::*;

    #[tokio::test]
    /// Test basic cia402 state transitions
    async fn home() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let gantry = Gantry::start(canopen, YZ_CONFIG).await?;

        // Create a task for the test logic
        let test_task = tokio::spawn(test_gantry(gantry));

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

    async fn test_gantry(gantry: Gantry) -> anyhow::Result<()> {
        info!("TEST: Homing gantry");

        gantry.send_command(GantryCommand::Home).await?;

        info!("TEST: wait on gantry homed");
        wait_until_gantry_command_completed(
            GantryCommand::Home,
            gantry.get_event_rx(),
            &gantry,
            &gantry.cfg,
            HOME_TIMEOUT,
        )
        .await?;
        info!("TEST: Gantry homed!");

        let target_x = Length::new::<millimeter>(0.5);
        let target_y = Length::new::<millimeter>(0.5);
        let target_z = Length::new::<millimeter>(0.5);
        let vel = Velocity::new::<meter_per_second>(TEST_VEL);

        let setpoint = GantryCommand::Setpoint {
            x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: target_x,
                velocity: vel,
            })),
            y: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: target_y,
                velocity: vel,
            })),
            z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: target_z,
                velocity: vel,
            })),
        };

        info!("TEST: Sending setpoint: {:?}", setpoint.clone());

        wait_until_gantry_command_completed(
            setpoint.clone(),
            gantry.get_event_rx(),
            &gantry,
            &gantry.cfg,
            TIMEOUT,
        )
        .await?;

        Ok(())
    }
}

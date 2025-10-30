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
                wait_for_position_target_reached, wait_for_target_reached, wait_until_event_matches,
            },
        },
        gantry::Gantry,
    };
    use tokio::signal;
    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::meter_per_second,
    };

    use gantry_demo::config::*;

    use crate::common::{HOME_TIMEOUT, TIMEOUT};

    use super::*;

    const TEST_SETPOINT_INITIAL: (f64, f64, f64) = (15.0, 10.0, 5.0);
    const TEST_VEL: f64 = 0.01;
    const TEST_SETPOINTS: [(f64, f64, f64); 4] = [
        (20.0, 15.0, 10.0),
        (10.0, 5.0, 10.0),
        (0.0, 10.0, 0.0),
        (10.0, 10.0, 5.0),
    ];
    const TEST_SETPOINTS_LEN: usize = TEST_SETPOINTS.len();

    #[tokio::test]
    /// Test basic cia402 state transitions
    async fn pos_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let gantry = Gantry::start(
            canopen,
            DEFAULT_X_CONFIG,
            DEFAULT_Y_CONFIG,
            DEFAULT_Z_CONFIG,
        )
        .await?;

        // Create a task for the test logic
        let test_task = tokio::spawn(test_gantry_pos_full(gantry));

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

    async fn test_gantry_pos_full(gantry: Gantry) -> anyhow::Result<()> {
        info!("TEST: Homing gantry");

        gantry.send_command(GantryCommand::Home).await?;

        info!("TEST: wait on gantry homed");
        tokio::try_join!(
            wait_for_target_reached(
                gantry.get_event_rx(),
                gantry_axis::event::util::TargetQuantity::Home(true),
                Axis::X,
                HOME_TIMEOUT,
            ),
            wait_for_target_reached(
                gantry.get_event_rx(),
                gantry_axis::event::util::TargetQuantity::Home(true),
                Axis::Y,
                HOME_TIMEOUT,
            ),
            wait_for_target_reached(
                gantry.get_event_rx(),
                gantry_axis::event::util::TargetQuantity::Home(true),
                Axis::Z,
                HOME_TIMEOUT,
            ),
        )?;
        info!("TEST: Gantry homed!");

        info!("Moving Gantry into initial test position");
        let vel = Velocity::new::<meter_per_second>(TEST_VEL);
        let target_x = Length::new::<millimeter>(TEST_SETPOINT_INITIAL.0);
        let target_y = Length::new::<millimeter>(TEST_SETPOINT_INITIAL.1);
        let target_z = Length::new::<millimeter>(TEST_SETPOINT_INITIAL.2);

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
        gantry.send_command(setpoint.clone()).await?;

        tokio::try_join!(
            wait_for_target_reached(
                gantry.get_event_rx(),
                gantry_axis::event::util::TargetQuantity::Position(target_x.get::<millimeter>()),
                Axis::X,
                TIMEOUT,
            ),
            wait_for_target_reached(
                gantry.get_event_rx(),
                gantry_axis::event::util::TargetQuantity::Position(target_y.get::<millimeter>()),
                Axis::Y,
                TIMEOUT,
            ),
            wait_for_target_reached(
                gantry.get_event_rx(),
                gantry_axis::event::util::TargetQuantity::Position(target_z.get::<millimeter>()),
                Axis::Z,
                TIMEOUT,
            ),
        )?;

        let pos_zero = Length::new::<millimeter>(0.0);

        for _num in 1..50 {
            for setpoint_idx in 0..TEST_SETPOINTS_LEN {
                let target_x = Length::new::<millimeter>(TEST_SETPOINTS[setpoint_idx].0);
                let target_y = Length::new::<millimeter>(TEST_SETPOINTS[setpoint_idx].1);
                let target_z = Length::new::<millimeter>(TEST_SETPOINTS[setpoint_idx].2);

                let event_rx = gantry.get_event_rx();
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

                gantry.send_command(setpoint.clone()).await?;

                tokio::try_join!(
                    wait_for_target_reached(
                        gantry.get_event_rx(),
                        gantry_axis::event::util::TargetQuantity::Position(
                            target_x.get::<millimeter>()
                        ),
                        Axis::X,
                        TIMEOUT,
                    ),
                    wait_for_target_reached(
                        gantry.get_event_rx(),
                        gantry_axis::event::util::TargetQuantity::Position(
                            target_y.get::<millimeter>()
                        ),
                        Axis::Y,
                        TIMEOUT,
                    ),
                    wait_for_target_reached(
                        gantry.get_event_rx(),
                        gantry_axis::event::util::TargetQuantity::Position(
                            target_z.get::<millimeter>()
                        ),
                        Axis::Z,
                        TIMEOUT,
                    ),
                )?;

                info!("TEST: setpoint: {:?} REACHED", setpoint.clone());
            }
        }

        Ok(())
    }
}

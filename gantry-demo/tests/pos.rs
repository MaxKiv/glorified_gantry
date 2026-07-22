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
            GantryMotorEventContent,
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

    use super::*;

    const TEST_SETPOINT_INITIAL: (f64, f64, f64) = (15.0, 25.0, 30.0);
    const TEST_VEL: f64 = 0.0001;
    const TEST_SETPOINTS: [(f64, f64, f64); 4] = [
        (20.0, 5.0, 5.0),
        (10.0, 50.0, 5.0),
        (0.0, 50.0, 80.0),
        (10.0, 5.0, 80.0),
    ];
    const TEST_SETPOINTS_LEN: usize = TEST_SETPOINTS.len();

    #[tokio::test]
    /// Test basic cia402 state transitions
    async fn pos_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let gantry = Gantry::start(canopen, YZ_CONFIG).await?;

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
        wait_until_gantry_command_completed(
            GantryCommand::Home,
            gantry.get_event_rx(),
            &gantry,
            HOME_TIMEOUT,
        )
        .await?;
        info!("TEST: Gantry homed!");

        sleep(Duration::from_millis(2000)).await;

        // info!("Moving Gantry into initial test position");
        let vel = Velocity::new::<meter_per_second>(TEST_VEL);

        // let target_x = Length::new::<millimeter>(TEST_SETPOINT_INITIAL.0);
        // let target_y = Length::new::<millimeter>(TEST_SETPOINT_INITIAL.1);
        // let target_z = Length::new::<millimeter>(TEST_SETPOINT_INITIAL.2);

        // let setpoint = GantryCommand::Setpoint {
        //     x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
        //         target: target_x,
        //         velocity: vel,
        //     })),
        //     y: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
        //         target: target_y,
        //         velocity: vel,
        //     })),
        //     z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
        //         target: target_z,
        //         velocity: vel,
        //     })),
        // };

        // info!("TEST: Sending setpoint: {:?}", setpoint.clone());

        // wait_until_gantry_command_completed(
        //     setpoint.clone(),
        //     gantry.get_event_rx(),
        //     &gantry,
        //     &gantry.cfg,
        //     TIMEOUT,
        // )
        // .await?;

        let pos_zero = Length::new::<millimeter>(0.0);

        for _num in 1..u64::MAX {
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

                wait_until_gantry_command_completed(setpoint.clone(), event_rx, &gantry, TIMEOUT)
                    .await?;
                info!("TEST: setpoint: {:?} REACHED", setpoint.clone());

                // sleep(Duration::from_millis(1000)).await;
            }
        }

        Ok(())
    }
}

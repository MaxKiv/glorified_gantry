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
                TargetQuantity, send_cmd_and_wait_until_gantry_command_completed,
                wait_for_axis_setpoint_complete, wait_until_event_matches,
            },
        },
        gantry::Gantry,
    };
    use std::time::Duration;
    use tokio::{signal, time::sleep};
    use uom::si::{
        f64::{Length, Velocity},
        length::{decimeter, millimeter},
        velocity::meter_per_second,
    };

    use gantry_demo::config::*;

    use crate::common::{HOME_TIMEOUT, TIMEOUT};

    use super::*;

    const TEST_SETPOINT_INITIAL: (f64, f64, f64) = (15.0, 25.0, 30.0);
    const TEST_VEL: f64 = 0.001;
    const TEST_SETPOINTS: [(f64, f64, f64); 4] = [
        (20.0, 5.0, 5.0),
        (10.0, 50.0, 5.0),
        (0.0, 50.0, 84.5),
        (10.0, 5.0, 84.5),
    ];
    const TEST_SETPOINTS_LEN: usize = TEST_SETPOINTS.len();

    #[tokio::test]
    /// Test basic cia402 state transitions
    async fn hannover_messe() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        info!("Starting gantry");
        let cfg = YZ_CONFIG;
        let gantry = Gantry::start(canopen, cfg).await?;

        // Home gantry
        send_cmd_and_wait_until_gantry_command_completed(
            GantryCommand::Home,
            gantry.get_event_rx(),
            &gantry,
            HOME_TIMEOUT,
        )
        .await?;
        info!("TEST: Gantry homed!");

        sleep(Duration::from_millis(2000)).await;

        test_gantry_hannover_messe(gantry).await?;

        Ok(())
    }

    async fn test_gantry_hannover_messe(gantry: Gantry) -> anyhow::Result<()> {
        info!("TEST: Homing gantry");

        gantry.send_command(GantryCommand::Home).await?;

        info!("TEST: wait on gantry homed");
        send_cmd_and_wait_until_gantry_command_completed(
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

                send_cmd_and_wait_until_gantry_command_completed(
                    setpoint,
                    gantry.get_event_rx(),
                    &gantry,
                    TIMEOUT,
                )
                .await?;

                // sleep(Duration::from_millis(666)).await;
            }
        }

        Ok(())
    }
}

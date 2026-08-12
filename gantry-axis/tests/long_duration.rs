pub mod common;

use std::time::Duration;

use gantry_axis::{command::GantryCommand, gantry::Gantry};
use tracing::*;

use crate::common::test_gantry_cmds;

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use gantry_axis::{
        axis::setpoint::{AxisSetpoint, PositionSetpoint},
        command::GantryCommand,
        event::util::send_cmd_and_wait_until_gantry_command_completed,
        gantry::Gantry,
    };

    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::meter_per_second,
    };

    use gantry_demo::config::TEST_CONFIG;

    use super::*;

    #[tokio::test]
    async fn long_duration_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let vel = Velocity::new::<meter_per_second>(0.01);
        let targets = [
            (10.0, 5.0, 30.0),
            (12.0, 0.0, 28.0),
            (14.0, 0.0, 26.0),
            (16.0, 0.0, 24.0),
            (18.0, 0.0, 22.0),
            (19.0, 5.0, 20.0),
            (22.0, 0.0, 18.0),
            (24.0, 0.0, 16.0),
            (26.0, 0.0, 14.0),
            (28.0, 0.0, 12.0),
        ];

        let mut cmds = vec![];
        for i in 0..targets.len() {
            cmds.push(GantryCommand::Setpoint {
                x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: Length::new::<millimeter>(targets[i].0),
                    velocity: vel,
                })),
                y: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: Length::new::<millimeter>(targets[i].1),
                    velocity: vel,
                })),
                z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: Length::new::<millimeter>(targets[i].2),
                    velocity: vel,
                })),
            });
        }

        let timeout = Duration::from_secs(4);
        let cfg = TEST_CONFIG;
        // let cfg = Z_ONLY_CONFIG;
        let gantry = Gantry::start(canopen, cfg).await?;

        if let Err(err) = send_cmd_and_wait_until_gantry_command_completed(
            GantryCommand::Home,
            gantry.get_event_rx(),
            &gantry,
            timeout,
        )
        .await
        {
            error!("Unable to home gantry");
            return Err(err);
        }

        // Wait for either Ctrl-C or test completion
        let out = tokio::select! {
            out = single_test_cycle(&gantry, &cmds, "LONG_DURATION_TEST", timeout.clone()) => {
                out
            },
            _ = tokio::signal::ctrl_c() => {
                return Err(anyhow::anyhow!("SIGINT Received"))
            },
        };

        gantry.wait_for_shutdown().await;

        out
    }
}

async fn single_test_cycle(
    gantry: &Gantry,
    cmds: &[GantryCommand],
    name: &str,
    timeout: Duration,
) -> anyhow::Result<()> {
    const NUM_TESTS: usize = 1000;
    for i in 0..NUM_TESTS {
        warn!("------- Starting test {i} -------");
        if let Err(err) = test_gantry_cmds(&gantry, &cmds, name, timeout).await {
            return Err(err);
        }
    }

    Ok(())
}

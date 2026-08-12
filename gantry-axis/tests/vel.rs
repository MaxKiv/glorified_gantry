pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use gantry_axis::{axis::setpoint::AxisSetpoint, command::GantryCommand, gantry::Gantry};

    use uom::si::f64::Velocity;

    use gantry_demo::config::TEST_CONFIG;

    use crate::common::test_gantry_cmds;

    use super::*;

    #[tokio::test]
    async fn vel_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let targets = [
            (3.0, 0.0, 3.0),
            (1.0, 0.0, 1.0),
            (-1.0, 0.0, -1.0),
            (1.0, 0.0, 1.0),
            (-1.0, 0.0, -1.0),
            (1.0, 0.0, 1.0),
            (-1.0, 0.0, -1.0),
            (1.0, 0.0, 1.0),
            (-1.0, 0.0, -1.0),
            (1.0, 0.0, 1.0),
            (-1.0, 0.0, -1.0),
            (1.0, 0.0, 1.0),
            (-1.0, 0.0, -1.0),
            (1.0, 0.0, 1.0),
            (-1.0, 0.0, -1.0),
        ];

        let mut cmds = vec![GantryCommand::Home];
        for i in 0..targets.len() {
            cmds.push(GantryCommand::Setpoint {
                x: Some(AxisSetpoint::Velocity(
                    gantry_axis::axis::setpoint::VelocitySetpoint {
                        target: Velocity::new::<uom::si::velocity::millimeter_per_second>(
                            targets[i].0,
                        ),
                    },
                )),
                y: Some(AxisSetpoint::Velocity(
                    gantry_axis::axis::setpoint::VelocitySetpoint {
                        target: Velocity::new::<uom::si::velocity::millimeter_per_second>(
                            targets[i].1,
                        ),
                    },
                )),
                z: Some(AxisSetpoint::Velocity(
                    gantry_axis::axis::setpoint::VelocitySetpoint {
                        target: Velocity::new::<uom::si::velocity::millimeter_per_second>(
                            targets[i].2,
                        ),
                    },
                )),
            });
        }

        let timeout = Duration::from_secs(4);
        let cfg = TEST_CONFIG;
        let gantry = Gantry::start(canopen, cfg).await?;

        // Wait for either Ctrl-C or test completion
        let out = tokio::select! {
            out = test_gantry_cmds(&gantry, &cmds, "Velocity", timeout)=> {
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

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

    use tokio::signal;
    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::meter_per_second,
    };

    use gantry_demo::config::{TEST_CONFIG, Z_ONLY_CONFIG};

    use crate::common::test_gantry_cmds;

    use super::*;

    #[tokio::test]
    async fn vel_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let cfg = TEST_CONFIG;
        let gantry = Gantry::start(canopen, cfg).await?;

        let vel = Velocity::new::<meter_per_second>(0.01);

        // let targets = [
        //     (10.0, 5.0, 10.0),
        //     (12.0, 0.0, 08.0),
        //     (14.0, 0.0, 06.0),
        //     (16.0, 0.0, 04.0),
        //     (18.0, 0.0, 02.0),
        //     (10.0, 5.0, 10.0),
        //     (12.0, 0.0, 08.0),
        //     (14.0, 0.0, 06.0),
        //     (16.0, 0.0, 04.0),
        //     (18.0, 0.0, 02.0),
        //     (10.0, 5.0, 10.0),
        //     (12.0, 0.0, 08.0),
        //     (14.0, 0.0, 06.0),
        //     (16.0, 0.0, 04.0),
        //     (18.0, 0.0, 02.0),
        //     (10.0, 5.0, 10.0),
        //     (12.0, 0.0, 08.0),
        //     (14.0, 0.0, 06.0),
        //     (16.0, 0.0, 04.0),
        //     (18.0, 0.0, 02.0),
        // ];

        let targets = [
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
                // x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                //     target: Length::new::<millimeter>(targets[i].0),
                //     velocity: vel,
                // })),
                // y: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                //     target: Length::new::<millimeter>(targets[i].1),
                //     velocity: vel,
                // })),
                // z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                //     target: Length::new::<millimeter>(targets[i].2),
                //     velocity: vel,
                // })),
            });
        }

        let timeout = Duration::from_secs(5);

        // Wait for either Ctrl-C or test completion
        tokio::select! {
            res = test_gantry_cmds(gantry, &cmds, "Velocity", timeout)=> {
                return res;
            }
            _ = signal::ctrl_c() => {
                info!("Ctrl-C received — aborting test");
                return Err(anyhow::anyhow!("aborted test"));
            }
        }
    }
}

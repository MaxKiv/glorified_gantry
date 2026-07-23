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

    use gantry_demo::config::TEST_CONFIG;

    use crate::common::test_gantry_cmds;

    use super::*;

    #[tokio::test]
    async fn pos_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let cfg = TEST_CONFIG;
        let gantry = Gantry::start(canopen, cfg).await?;

        let vel = Velocity::new::<meter_per_second>(0.01);
        let pos_targets = [
            (10.0, 5.0, 10.0),
            (20.0, 0.0, 20.0),
            (10.0, 5.0, 10.0),
            (20.0, 0.0, 20.0),
        ];

        let mut cmds = vec![GantryCommand::Home];
        for i in 0..pos_targets.len() {
            cmds.push(GantryCommand::Setpoint {
                x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: Length::new::<millimeter>(pos_targets[i].0),
                    velocity: vel,
                })),
                y: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: Length::new::<millimeter>(pos_targets[i].0),
                    velocity: vel,
                })),
                z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: Length::new::<millimeter>(pos_targets[i].0),
                    velocity: vel,
                })),
            });
        }

        let timeout = Duration::from_secs(10);
        test_gantry_cmds(gantry, &cmds, "Position", timeout).await?;

        Ok(())
    }
}

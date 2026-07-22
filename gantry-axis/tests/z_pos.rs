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
        event::util::{HOME_TIMEOUT, wait_for_target_reached, wait_until_gantry_homed},
        gantry::Gantry,
        setpoint::translator::scaling::DeviceScaling,
    };
    use tokio::{signal, time::sleep};
    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::meter_per_second,
    };

    use gantry_demo::config::{TEST_X_CONFIG, TEST_Y_CONFIG, TEST_Z_CONFIG, Z_ONLY_CONFIG};

    use crate::common::{TIMEOUT, test_gantry_cmds};

    use super::*;

    #[tokio::test]
    async fn z_pos_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let cfg = Z_ONLY_CONFIG;
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

        // Wait for either Ctrl-C or test completion
        tokio::select! {
            res = test_gantry_cmds(gantry, &cmds, "Z position") => {
                res?;
            }
            _ = signal::ctrl_c() => {
                info!("Ctrl-C received — aborting test");
            }
        }

        Ok(())
    }
}

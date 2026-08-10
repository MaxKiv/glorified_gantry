pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Context;
    use gantry_axis::{
        axis::{
            Axis,
            setpoint::{AxisSetpoint, PositionSetpoint, TorqueSetpoint},
        },
        command::GantryCommand,
        event::util::{
            HOME_TIMEOUT, send_cmd_and_wait_until_gantry_command_completed,
            wait_for_axis_setpoint_complete,
        },
        gantry::Gantry,
        setpoint::translator::scaling::DeviceScaling,
    };
    use tokio::{signal, time::sleep};
    use uom::si::{
        f64::{Length, Torque, Velocity},
        length::millimeter,
        torque::newton_meter,
        velocity::meter_per_second,
    };

    use gantry_demo::config::{TEST_CONFIG, TEST_X_CONFIG, TEST_Y_CONFIG, TEST_Z_CONFIG};

    use crate::common::{TIMEOUT, test_gantry_cmds};

    use super::*;

    #[tokio::test]
    async fn torque_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let cfg = TEST_CONFIG;
        let gantry = Gantry::start(canopen, cfg).await?;

        let vel = Velocity::new::<meter_per_second>(0.01);
        let tau_targets = [
            (0.1, -0.1, 0.1),
            (-0.1, 0.1, -0.1),
            (0.1, -0.1, 0.1),
            (-0.1, 0.1, -0.1),
            (0.1, -0.1, 0.1),
            (-0.1, 0.1, -0.1),
            (0.1, -0.1, 0.1),
            (-0.1, 0.1, -0.1),
        ];

        let mut cmds = vec![
            // Home first
            GantryCommand::Home,
            // Move to save position
            GantryCommand::Setpoint {
                x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: Length::new::<millimeter>(10.0),
                    velocity: vel,
                })),
                y: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: Length::new::<millimeter>(10.0),
                    velocity: vel,
                })),
                z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                    target: Length::new::<millimeter>(10.0),
                    velocity: vel,
                })),
            },
        ];

        // Cycle through torques
        for i in 0..tau_targets.len() {
            cmds.push(GantryCommand::Setpoint {
                x: Some(AxisSetpoint::Torque(TorqueSetpoint {
                    target: Torque::new::<newton_meter>(tau_targets[i].0),
                })),
                y: Some(AxisSetpoint::Torque(TorqueSetpoint {
                    target: Torque::new::<newton_meter>(tau_targets[i].0),
                })),
                z: Some(AxisSetpoint::Torque(TorqueSetpoint {
                    target: Torque::new::<newton_meter>(tau_targets[i].0),
                })),
            });
        }

        let timeout = Duration::from_secs(10);
        tokio::select! {
            out = test_gantry_cmds(&gantry, &cmds, "Torque", timeout)=> {
                out.context("test failed")
            },
            _ = tokio::signal::ctrl_c() => {
                gantry.wait_for_shutdown().await;
                Err(anyhow::anyhow!("SIGINT Received"))
            },
        }
    }
}

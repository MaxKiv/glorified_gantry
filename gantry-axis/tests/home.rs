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
        cfg::GantryConfig,
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

    use gantry_demo::config::{
        TEST_CONFIG, TEST_X_CONFIG, TEST_Y_CONFIG, TEST_Z_CONFIG, X_DISABLED, Y_DISABLED,
    };

    use crate::common::{TIMEOUT, test_gantry_cmds};

    use super::*;

    #[tokio::test]
    async fn home_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let cfg = TEST_CONFIG;
        let gantry = Gantry::start(canopen, cfg).await?;

        let cmds = vec![
            // Home first
            GantryCommand::Home,
        ];

        // Wait for either Ctrl-C or test completion
        tokio::select! {
            res = test_gantry_cmds(gantry, &cmds, "Homing") => {
                res?;
            }
            _ = signal::ctrl_c() => {
                info!("Ctrl-C received — aborting test");
            }
        }

        Ok(())
    }
}

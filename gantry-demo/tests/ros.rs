pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    use gantry_axis::{
        axis::setpoint::{AxisSetpoint, PositionSetpoint},
        command::GantryCommand,
        event::{
            GantryEvent,
            util::{
                wait_for_position_target_reached, wait_for_target_reached,
                wait_until_event_matches, wait_until_gantry_command_completed,
            },
        },
        gantry::Gantry,
    };
    use gantry_ros2::bridge::run_gantry_ros_bridge;
    use std::time::Duration;
    use tokio::{
        signal,
        sync::{broadcast, mpsc, watch},
        task::JoinHandle,
        time::sleep,
    };
    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::meter_per_second,
    };

    use gantry_demo::config::*;

    use crate::common::{HOME_TIMEOUT, TIMEOUT};

    use super::*;

    const TEST_SETPOINT_INITIAL: (f64, f64, f64) = (10.0, 10.0, 10.0);
    const TEST_VEL: f64 = 0.01;

    #[tokio::test]
    /// Test basic cia402 state transitions
    async fn ros_demo() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let gantry = Gantry::start(canopen, YZ_CONFIG).await?;

        // Create a task for the test logic
        let test_task = tokio::spawn(test_gantry_ros(gantry));

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

    async fn test_gantry_ros(gantry: Gantry) -> anyhow::Result<()> {
        info!("TEST: Homing gantry");
        info!("TEST: wait on gantry homed");
        wait_until_gantry_command_completed(
            GantryCommand::Home,
            gantry.get_event_rx(),
            &gantry,
            &gantry.cfg,
            HOME_TIMEOUT,
        )
        .await?;
        info!("TEST: Gantry homed!");

        info!("Moving Gantry into initial test position");
        let vel = Velocity::new::<meter_per_second>(TEST_VEL);
        let target_x = Length::new::<millimeter>(TEST_SETPOINT_INITIAL.0);
        let target_y = Length::new::<millimeter>(TEST_SETPOINT_INITIAL.1);
        let target_z = Length::new::<millimeter>(TEST_SETPOINT_INITIAL.2);
        let pos_zero = Length::new::<millimeter>(0.0);

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

        info!("TEST: Sending gantry to starting position: {setpoint:?}",);

        wait_until_gantry_command_completed(
            setpoint,
            gantry.get_event_rx(),
            &gantry,
            &gantry.cfg,
            TIMEOUT,
        )
        .await?;

        info!("TEST: Starting position reached!");

        info!("Spawn the ROS2 bridge");
        let (bridge_handle, shutdown_bridge) =
            spawn_ros_bridge(gantry.get_event_rx(), gantry.get_cmd_tx());

        info!("TEST: Sleeping forever...");
        std::future::pending::<()>().await;
        // info!("TEST: Sleeping for 10 minutes");
        // sleep(Duration::from_secs(6000)).await;

        Ok(())
    }

    fn spawn_ros_bridge(
        rx: broadcast::Receiver<GantryEvent>,
        tx: mpsc::Sender<GantryCommand>,
    ) -> (JoinHandle<()>, watch::Sender<bool>) {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

        let bridge_handle = gantry_demo::spawn_logged("ROS", async move {
            tokio::select! {
                res = run_gantry_ros_bridge(rx, tx) => res,
                _ = shutdown_rx.changed() => {
                    info!("Shutdown signal received — stopping ROS bridge");
                    Ok(())
                }
            }
        });

        (bridge_handle, shutdown_tx)
    }
}

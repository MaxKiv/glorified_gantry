use tracing::*;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use gantry_axis::{
        axis::setpoint::{AxisSetpoint, PositionSetpoint},
        command::GantryCommand,
        event::{GantryMotorEvent, GantryMotorEventContent},
        gantry::Gantry,
        setpoint::translator::scaling::DeviceScaling,
    };
    use gantry_demo::config::*;
    use gantry_ros2::{bridge::run_gantry_ros_bridge, spawn_logged};
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

    use super::*;

    #[tokio::test]
    /// Test basic cia402 state transitions
    async fn test_spawn_ros_bridge() -> anyhow::Result<()> {
        // Set up logging library
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        info!("Start Gantry, this initializes all axis motors using the given configs");
        let gantry = Gantry::start(canopen, TEST_CONFIG).await?;

        info!("Spawn the ROS2 bridge");
        let (bridge_handle, shutdown_bridge) =
            spawn_ros_bridge(gantry.get_event_rx(), gantry.get_cmd_tx());

        // Create a task for the test logic
        info!("Starting test");
        let test_task = tokio::spawn(test_ros_bridge(gantry));

        // Wait for either Ctrl-C or test completion
        tokio::select! {
            res = test_task => {
                res??;
            }
            _ = signal::ctrl_c() => {
                let _ = shutdown_bridge.send(true);
                info!("Ctrl-C received — aborting test");
            }
        }

        let _ = bridge_handle.await;

        Ok(())
    }

    fn spawn_ros_bridge(
        rx: broadcast::Receiver<GantryMotorEvent>,
        tx: mpsc::Sender<GantryCommand>,
    ) -> (JoinHandle<()>, watch::Sender<bool>) {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

        let bridge_handle = spawn_logged("ROS", async move {
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

    async fn test_ros_bridge(gantry: Gantry) -> anyhow::Result<()> {
        gantry.send_command(GantryCommand::Home).await?;

        sleep(Duration::from_secs(600)).await;

        Ok(())
    }
}

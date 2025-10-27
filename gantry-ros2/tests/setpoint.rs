use tracing::*;

#[cfg(test)]
mod tests {
    use r2r::geometry_msgs::msg::Vector3;
    use std::time::Duration;

    use gantry_axis::{
        axis::setpoint::{AxisSetpoint, PositionSetpoint},
        command::GantryCommand,
        event::GantryEvent,
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
    async fn test_ros_bridge_setpoint() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let gantry = Gantry::start(
            canopen,
            TEST_X_CONFIG,
            TEST_Y_CONFIG,
            TEST_Z_CONFIG,
            DeviceScaling::test_setup(),
        )
        .await?;

        let (bridge_handle, shutdown_bridge) =
            spawn_ros_bridge(gantry.get_event_rx(), gantry.get_cmd_tx());

        // Create a task for the test logic
        let test_task = tokio::spawn(test_ros_bridge_setpoint_logic(gantry));

        // Wait for either Ctrl-C or test completion
        tokio::select! {
            res = test_task => {
                res??;
            }
            _ = signal::ctrl_c() => {
                info!("Ctrl-C received — aborting test");
            }
            _ = tokio::time::sleep(Duration::from_secs(20)) => {
                info!("Timeout reached — aborting test");
            }
        }

        let _ = shutdown_bridge.send(true);
        let _ = bridge_handle.await;

        Ok(())
    }

    fn spawn_ros_bridge(
        rx: broadcast::Receiver<GantryEvent>,
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

    async fn test_ros_bridge_setpoint_logic(gantry: Gantry) -> anyhow::Result<()> {
        gantry.send_command(GantryCommand::Home).await?;

        // Setpoint
        let mut msg = Vector3 {
            x: 30.0,
            y: 10.0,
            z: 10.0,
        };

        info!("Publishing setpoint: {msg:?} to /setpoint/position");

        publish_test_setpoint(msg.clone()).await?;

        sleep(Duration::from_secs(3)).await;

        msg.x = -msg.x;
        msg.y = -msg.y;
        msg.z = -msg.z;
        info!("Publishing setpoint: {msg:?} to /setpoint/position");
        publish_test_setpoint(msg).await?;

        sleep(Duration::from_secs(3)).await;

        Ok(())
    }

    pub async fn publish_test_setpoint(msg: Vector3) -> anyhow::Result<()> {
        // Create ROS2 context and node
        let ctx = r2r::Context::create()?;
        let mut node = r2r::Node::create(ctx, "test_setpoint_publisher", "")?;

        // Create publisher
        let publisher =
            node.create_publisher::<Vector3>("/setpoint/position", r2r::QosProfile::default())?;

        // Publish once
        publisher.publish(&msg)?;
        println!("Published test setpoint: {:?}", msg);

        // Give ROS2 some time to deliver the message
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(())
    }
}

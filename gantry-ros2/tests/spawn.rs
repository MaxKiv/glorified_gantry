use tracing::*;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use gantry_axis::{
        axis::setpoint::{AxisSetpoint, PositionSetpoint},
        command::GantryCommand,
        event::GantryEvent,
        gantry::Gantry,
    };
    use gantry_demo::config::*;
    use gantry_ros2::{bridge::run_gantry_ros_bridge, spawn_logged};
    use tokio::{signal, sync::broadcast, task::JoinHandle, time::sleep};
    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::meter_per_second,
    };

    use super::*;

    #[tokio::test]
    /// Test basic cia402 state transitions
    async fn test_spawn_ros_bridge() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let gantry = Gantry::start(canopen, TEST_X_CONFIG, TEST_Y_CONFIG, TEST_Z_CONFIG).await?;

        let bridge_handle = spawn_ros_bridge(gantry.get_event_rx());

        // Create a task for the test logic
        let test_task = tokio::spawn(test_ros_bridge(gantry));

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

    fn spawn_ros_bridge(rx: broadcast::Receiver<GantryEvent>) -> JoinHandle<()> {
        spawn_logged("ROS", async move {
            // Spin the ROS executor, until program exits or Ctrl-C is received
            tokio::select! {
                res = run_gantry_ros_bridge(rx) => {res}
                _ = signal::ctrl_c() => {
                    info!("Ctrl-C received — stopping ROS2 Bridge");
                    Ok(())
                }
            }
        })
    }

    async fn test_ros_bridge(gantry: Gantry) -> anyhow::Result<()> {
        gantry.send_command(GantryCommand::Home).await?;

        sleep(Duration::from_secs(2)).await;

        let setpoint = GantryCommand::Setpoint {
            x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: Length::new::<millimeter>(10.0),
                velocity: Velocity::new::<meter_per_second>(0.01),
            })),
            y: None,
            z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: Length::new::<millimeter>(10.0),
                velocity: Velocity::new::<meter_per_second>(0.01),
            })),
        };

        gantry.send_command(setpoint).await?;

        sleep(Duration::from_secs(5)).await;

        let setpoint = GantryCommand::Setpoint {
            x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: Length::new::<millimeter>(-40.0),
                velocity: Velocity::new::<meter_per_second>(0.01),
            })),
            y: None,
            z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: Length::new::<millimeter>(-20.0),
                velocity: Velocity::new::<meter_per_second>(0.01),
            })),
        };

        gantry.send_command(setpoint).await?;

        sleep(Duration::from_secs(5)).await;

        Ok(())
    }
}

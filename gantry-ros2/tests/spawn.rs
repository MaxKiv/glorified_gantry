pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::StreamExt;
    use gantry_axis::{
        axis::setpoint::{AxisSetpoint, PositionSetpoint},
        command::GantryCommand,
        event::util::{
                HOME_TIMEOUT, TIMEOUT, send_cmd_and_wait_until_gantry_command_completed,
                wait_until_cmd_completed,
            },
        gantry::Gantry,
    };
    use gantry_demo::config::*;
    use gantry_ros2::bridge::Ros2Bridge;
    use r2r::geometry_msgs::msg::Vector3;
    
    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::meter_per_second,
    };

    use crate::common::*;

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

        info!("Home Gantry");
        send_cmd_and_wait_until_gantry_command_completed(
            GantryCommand::Home,
            gantry.get_event_rx(),
            &gantry,
            HOME_TIMEOUT,
        )
        .await?;
        info!("Gantry homed!");

        info!("Spawning ROS2 bridge");
        let ros2_bridge =
            Ros2Bridge::try_spawn_ros_bridge(gantry.get_event_rx(), gantry.get_cmd_tx())?;

        info!("Spawning ROS2 test node");
        let ros2_test_node = Ros2TestNode::try_spawn_ros_test_node()?;

        info!("Testing ROS2 bridge");
        test_ros_bridge(gantry, ros2_bridge, ros2_test_node).await?;

        info!("ROS test success, shuting down...");
        Ok(())
    }

    async fn test_ros_bridge(
        gantry: Gantry,
        _ros2_bridge: Ros2Bridge,
        mut ros2_test_node: Ros2TestNode,
    ) -> anyhow::Result<()> {
        // Check if JointState messages come in
        for _i in 0..10 {
            let js = ros2_test_node
                .joint_sub
                .next()
                .await
                .ok_or(anyhow::anyhow!("No jointstate message received"))?;
            info!("Got JointState: {:?}", js);
        }

        // Check position setpoint
        let pos = Vector3 {
            x: 10.4,
            y: 11.5,
            z: 12.6,
        };
        info!(
            "Testing position setpoint ({:?}) propagation through ROS2 bridge",
            pos
        );
        let vel = Velocity::new::<meter_per_second>(0.01);
        ros2_test_node.pos_pub.publish(&pos)?;
        let cmd = GantryCommand::Setpoint {
            x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: Length::new::<millimeter>(pos.x),
                velocity: vel,
            })),
            y: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: Length::new::<millimeter>(pos.y),
                velocity: vel,
            })),
            z: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
                target: Length::new::<millimeter>(pos.z),
                velocity: vel,
            })),
        };

        info!("Waiting for Gantry to arrive at {:?}", cmd);
        wait_until_cmd_completed(cmd.clone(), gantry.get_event_rx(), &gantry, TIMEOUT).await?;
        info!("Gantry indicates {:?} reached!", cmd);

        // Check jointstate
        info!("Checking published jointstate also reflects correct state");
        tokio::time::timeout(
            Duration::from_secs(1),
            check_jointstate(ros2_test_node, pos),
        )
        .await??;

        Ok(())
    }

    fn pos_within_range(curr: f64, target: f64) -> bool {
        const EPSILON: f64 = 0.1;
        (curr - target).abs() <= EPSILON
    }

    async fn check_jointstate(
        mut ros2_test_node: Ros2TestNode,
        target: Vector3,
    ) -> anyhow::Result<()> {
        loop {
            let js = ros2_test_node
                .joint_sub
                .next()
                .await
                .ok_or(anyhow::anyhow!("No jointstate message received"))?;
            info!("Got JointState: {:?}", js);

            if pos_within_range(js.position[0], target.x)
                && pos_within_range(js.position[1], target.y)
                && pos_within_range(js.position[2], target.z)
            {
                info!("SUCCESS: JointState {:?} is within range", js);
                return Ok(());
            }
        }
    }
}

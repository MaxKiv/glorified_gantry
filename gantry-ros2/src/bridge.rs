use anyhow::Context;
use gantry_axis::command::GantryCommand;
use gantry_axis::event::GantryMotorEvent;
use r2r::geometry_msgs::msg::Vector3;
use r2r::{self, QosProfile, diagnostic_msgs, sensor_msgs};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;
use tracing::*;

use crate::events::bridge_gantry_events;
use crate::setpoints::bridge_gantry_setpoints;

pub const EXECUTOR_SPIN_PERIOD: Duration = Duration::from_millis(10);

/// ROS2 setpoint & joint state Bridge
/// NOTE: aborts ROS2 node on [`Drop`]
pub struct Ros2Bridge {
    _tasks: JoinSet<anyhow::Result<()>>,
}

impl Ros2Bridge {
    pub fn try_spawn_ros_bridge(
        rx: broadcast::Receiver<GantryMotorEvent>,
        tx: mpsc::Sender<GantryCommand>,
    ) -> anyhow::Result<Self> {
        info!("running gantry ros bridge");

        let ctx = r2r::Context::create().context("Failed to create ROS2 context")?;
        const NODE_NAME: &str = "gantry_bridge";
        let mut node =
            r2r::Node::create(ctx, NODE_NAME, "").context("Failed to create ROS2 Node")?;

        info!("Creating /joint_states publisher");
        let joint_pub = node.create_publisher::<sensor_msgs::msg::JointState>(
            "/joint_states",
            r2r::QosProfile::default(),
        )?;

        info!("Creating /diagnostics publisher");
        let diag_pub = node.create_publisher::<diagnostic_msgs::msg::DiagnosticArray>(
            "/diagnostics",
            r2r::QosProfile::default(),
        )?;

        // Create subscribers
        info!("Creating /setpoint/position subscribers");
        let pos_sub = node
            .subscribe::<Vector3>("/setpoint/position", QosProfile::default())
            .context("Failed to create position subscribers")?;

        info!("Creating /setpoint/velocity subscribers");
        let vel_sub = node
            .subscribe::<Vector3>("/setpoint/velocity", QosProfile::default())
            .context("Failed to create velocity subscribers")?;

        info!("Creating /setpoint/torque subscribers");
        let torque_sub = node
            .subscribe::<Vector3>("/setpoint/torque", QosProfile::default())
            .context("Failed to create torque subscribers")?;

        info!("Spawning ROS2 Executor");
        let mut tasks = JoinSet::new();
        let _executor = tasks.spawn_blocking(move || {
            // Early spin in an attempt to make sure the executor is up before exiting this function
            // Its kinda jank, but so is the entirety of ROS 🤷
            node.spin_once(std::time::Duration::from_millis(10));
            loop {
                node.spin_once(EXECUTOR_SPIN_PERIOD);
            }
        });
        let _event_bridge = tasks.spawn(bridge_gantry_events(rx, joint_pub, diag_pub));
        let _setpoint_bridge =
            tasks.spawn(bridge_gantry_setpoints(tx, pos_sub, vel_sub, torque_sub));

        info!("ROS2 bridge node '{}' initialized", NODE_NAME);

        let bridge = Ros2Bridge { _tasks: tasks };

        Ok(bridge)
    }
}

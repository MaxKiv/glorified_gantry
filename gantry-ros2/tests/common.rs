use anyhow::Context;
use r2r::geometry_msgs::msg::Vector3;
use r2r::sensor_msgs::msg::JointState;
use r2r::{self, Publisher, QosProfile};
use tokio::task::JoinSet;
use tracing::*;

/// ROS2 Node to test [`Ros2Bridge`]
/// NOTE: aborts ROS2 node on [`Drop`]
pub struct Ros2TestNode {
    _tasks: JoinSet<anyhow::Result<()>>,
    pub pos_pub: Publisher<Vector3>,
    pub vel_pub: Publisher<Vector3>,
    pub tor_pub: Publisher<Vector3>,
    pub joint_sub: Box<dyn futures::Stream<Item = JointState> + Unpin>,
}

impl Ros2TestNode {
    pub fn try_spawn_ros_test_node() -> anyhow::Result<Self> {
        info!("spawning ROS2 test node");

        const NODE_NAME: &str = "Gantry ROS2 Test Node";
        let ctx = r2r::Context::create().context("Failed to create ROS2 context")?;
        let mut node =
            r2r::Node::create(ctx, NODE_NAME, "").context("Failed to create ROS2 Test Node")?;

        // Create subscribers
        info!("Creating /setpoint/position publisher");
        let pos_pub = node
            .create_publisher::<Vector3>("/setpoint/position", QosProfile::default())
            .context("Failed to create position subscribers")?;
        info!("Creating /setpoint/velocity publisher");
        let vel_pub = node
            .create_publisher::<Vector3>("/setpoint/velocity", QosProfile::default())
            .context("Failed to create position subscribers")?;
        info!("Creating /setpoint/torque publisher");
        let tor_pub = node
            .create_publisher::<Vector3>("/setpoint/torque", QosProfile::default())
            .context("Failed to create position subscribers")?;

        info!("Creating /joint_states subscribers");
        let joint_sub =
            node.subscribe::<JointState>("/joint_states", r2r::QosProfile::default())?;

        info!("Spawning ROS2 Executor");
        let mut tasks = JoinSet::new();
        let _executor = tasks.spawn_blocking(move || {
            // Early spin in an attempt to make sure the executor is up before exiting this function
            // Its kinda jank, but so is the entirety of ROS 🤷
            node.spin_once(std::time::Duration::from_millis(10));
            loop {
                node.spin_once(gantry_ros2::bridge::EXECUTOR_SPIN_PERIOD);
            }
        });

        info!("ROS2 test node '{}' initialized", NODE_NAME);

        let bridge = Ros2TestNode {
            _tasks: tasks,
            pos_pub,
            vel_pub,
            tor_pub,
            joint_sub: Box::new(joint_sub),
        };

        Ok(bridge)
    }
}

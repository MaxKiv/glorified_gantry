use anyhow::Context;
use gantry_axis::command::GantryCommand;
use gantry_axis::{diagnostic::DiagnosticLevel, event::GantryEvent};
use r2r::geometry_msgs::msg::Vector3;
use r2r::{self, QosProfile, diagnostic_msgs, sensor_msgs};
use std::time::Duration;
use tokio::signal;
use tokio::sync::{broadcast, mpsc};
use tracing::*;

use crate::events::bridge_gantry_events;
use crate::setpoints::bridge_gantry_setpoints;

const EXECUTOR_SPIN_PERIOD: Duration = Duration::from_millis(100);

pub async fn run_gantry_ros_bridge(
    rx: broadcast::Receiver<GantryEvent>,
    tx: mpsc::Sender<GantryCommand>,
) -> anyhow::Result<()> {
    info!("running gantry ros bridge");

    let ctx = r2r::Context::create().context("Failed to create ROS2 context")?;
    const NODE_NAME: &str = "gantry_bridge";
    let mut node = r2r::Node::create(ctx, NODE_NAME, "").context("Failed to create ROS2 Node")?;

    info!("Creating /joint_states publisher");
    let joint_pub = node.create_publisher::<sensor_msgs::msg::JointState>(
        "/joint_states",
        r2r::QosProfile::sensor_data(),
    )?;

    info!("Creating /diagnostics publisher");
    let diag_pub = node.create_publisher::<diagnostic_msgs::msg::DiagnosticArray>(
        "/diagnostics",
        r2r::QosProfile::default(),
    )?;

    // Create subscribers
    info!("Creating /setpoint/position publisher");
    let pos_sub = node
        .subscribe::<Vector3>("/setpoint/position", QosProfile::default())
        .context("Failed to create position publisher")?;

    info!("Creating /setpoint/velocity publisher");
    let vel_sub = node
        .subscribe::<Vector3>("/setpoint/velocity", QosProfile::default())
        .context("Failed to create velocity publisher")?;

    info!("Creating /setpoint/torque publisher");
    let torque_sub = node
        .subscribe::<Vector3>("/setpoint/torque", QosProfile::default())
        .context("Failed to create torque publisher")?;

    info!("Spawning ROS2 Executor");
    // Early spin in an attempt to make sure the executor is up before exiting this function
    // Its kinda jank, but so is the entirety of ROS 🤷
    node.spin_once(std::time::Duration::from_millis(1));
    let _executor = tokio::task::spawn_blocking(move || {
        loop {
            node.spin_once(EXECUTOR_SPIN_PERIOD);
        }
    });

    let events = tokio::task::spawn(bridge_gantry_events(rx, joint_pub, diag_pub));
    let setpoints = tokio::task::spawn(bridge_gantry_setpoints(tx, pos_sub, vel_sub, torque_sub));

    info!("ROS2 node '{}' initialized", NODE_NAME);

    tokio::select! {
        res = events => {
            if let Err(e) = res {
                error!("bridge_gantry_events task failed: {e:?}");
                anyhow::bail!("bridge_gantry_events task failed: {e:?}");
            } else {
                warn!("bridge_gantry_events task ended unexpectedly");
                Ok(())
            }
        }

        res = setpoints => {
            if let Err(e) = res {
                error!("bridge_gantry_setpoints task failed: {e:?}");
                anyhow::bail!("bridge_gantry_setpoints task failed: {e:?}");
            } else {
                warn!("bridge_gantry_setpoints task ended unexpectedly");
                Ok(())
            }
        }

        _ = signal::ctrl_c() => {
            info!("Ctrl-C received — stopping ROS2 Bridge");
            Ok(())
        }
    }
}

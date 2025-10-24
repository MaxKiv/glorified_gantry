use std::time::Duration;

use gantry_axis::{diagnostic::DiagnosticLevel, event::GantryEvent};
use r2r::{self, diagnostic_msgs, sensor_msgs};
use tokio::sync::broadcast;
use tracing::*;

const EXECUTOR_SPIN_PERIOD: Duration = Duration::from_millis(100);

pub async fn run_gantry_ros_bridge(mut rx: broadcast::Receiver<GantryEvent>) -> anyhow::Result<()> {
    info!("running gantry ros bridge");

    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "gantry_bridge", "")?;

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

    info!("Spawning ROS2 Executor");
    // Early spin in an attempt to make sure the executor is up before exiting this function
    // Its kinda jank, but so is the entirety of ROS 🤷
    node.spin_once(std::time::Duration::from_millis(10));
    let _executor = tokio::spawn(async move {
        loop {
            node.spin_once(std::time::Duration::from_millis(10));
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    });

    info!("Entering main loop");
    while let Ok(event) = rx.recv().await {
        match event {
            // Publish axis position feedback
            GantryEvent::Position { axis, value } => {
                let msg = sensor_msgs::msg::JointState {
                    name: vec![format!("{:?}", axis)],
                    position: vec![value],
                    ..Default::default()
                };

                trace!("Publishing /joint_states: {msg:?}");
                joint_pub.publish(&msg)?;
            }
            // Publish axis diagnostics
            GantryEvent::Diagnostic {
                axis,
                level,
                message,
            } => {
                let msg = diagnostic_msgs::msg::DiagnosticArray {
                    status: vec![diagnostic_msgs::msg::DiagnosticStatus {
                        name: format!("{:?}", axis),
                        message,
                        level: match level {
                            DiagnosticLevel::Ok => 0,
                            DiagnosticLevel::Warn => 1,
                            DiagnosticLevel::Error => 2,
                        },
                        ..Default::default()
                    }],
                    ..Default::default()
                };

                trace!("Publishing /diagnostics: {msg:?}");
                diag_pub.publish(&msg)?;
            }
            _ => {}
        }
    }

    Ok(())
}

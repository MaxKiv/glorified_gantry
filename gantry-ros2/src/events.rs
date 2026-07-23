use gantry_axis::{
    diagnostic::DiagnosticLevel,
    event::{GantryMotorEvent, GantryMotorEventContent},
};
use r2r::{self, Publisher, diagnostic_msgs, sensor_msgs};
use tokio::sync::broadcast;
use tracing::*;

pub async fn bridge_gantry_events(
    mut rx: broadcast::Receiver<GantryMotorEvent>,
    joint_pub: Publisher<sensor_msgs::msg::JointState>,
    diag_pub: Publisher<diagnostic_msgs::msg::DiagnosticArray>,
) -> anyhow::Result<()> {
    loop {
        match rx.recv().await {
            Ok(event) => {
                let GantryMotorEvent {
                    motor,
                    axis: _,
                    content,
                } = event;

                match content {
                    // Publish axis position feedback
                    GantryMotorEventContent::Position { value } => {
                        let msg = sensor_msgs::msg::JointState {
                            name: vec![format!("{:?}", motor)],
                            position: vec![value],
                            ..Default::default()
                        };

                        trace!("Publishing /joint_states: {msg:?}");
                        joint_pub.publish(&msg)?;
                    }

                    GantryMotorEventContent::Velocity { value } => {
                        let msg = sensor_msgs::msg::JointState {
                            name: vec![format!("{:?}", motor)],
                            velocity: vec![value],
                            ..Default::default()
                        };

                        trace!("Publishing /joint_states: {msg:?}");
                        joint_pub.publish(&msg)?;
                    }

                    GantryMotorEventContent::Torque { value } => {
                        let msg = sensor_msgs::msg::JointState {
                            name: vec![format!("{:?}", motor)],
                            effort: vec![value],
                            ..Default::default()
                        };

                        trace!("Publishing /joint_states: {msg:?}");
                        joint_pub.publish(&msg)?;
                    }

                    // Publish axis diagnostics
                    GantryMotorEventContent::Diagnostic { level, content } => {
                        let msg = diagnostic_msgs::msg::DiagnosticArray {
                            status: vec![diagnostic_msgs::msg::DiagnosticStatus {
                                name: format!("{:?}", motor),
                                message: format!("{:?}", content),
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
            Err(err) => match err {
                broadcast::error::RecvError::Closed => {
                    error!(
                        "Gantry Event receiver closed; gantry likely shut down. Shutting down ROS bridge..."
                    );
                    break;
                }

                broadcast::error::RecvError::Lagged(_) => error!(
                    "Gantry -> ROS event bridge lagged; indicates system overload. Attempting to continue..."
                ),
            },
        }
    }

    Ok(())
}

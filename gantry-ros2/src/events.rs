use gantry_axis::{diagnostic::DiagnosticLevel, event::GantryEvent};
use r2r::{self, Publisher, diagnostic_msgs, sensor_msgs};
use tokio::sync::broadcast;
use tracing::*;

pub async fn bridge_gantry_events(
    mut rx: broadcast::Receiver<GantryEvent>,
    joint_pub: Publisher<sensor_msgs::msg::JointState>,
    diag_pub: Publisher<diagnostic_msgs::msg::DiagnosticArray>,
) -> anyhow::Result<()> {
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

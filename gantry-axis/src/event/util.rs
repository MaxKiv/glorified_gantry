use anyhow::bail;
use tokio::{
    sync::broadcast,
    time::{self, Duration, Instant},
};
use tracing::*;

use crate::{axis::Axis, event::GantryEvent};

#[derive(Debug, Clone)]
pub enum TargetQuantity {
    Home(bool),
    Position(f64),
    Velocity(f64),
    Torque(f64),
}

/// Waits until a target is reached for the given axis
/// Note: The accuracy window
pub async fn wait_for_target_reached(
    event_rx: broadcast::Receiver<GantryEvent>,
    target: TargetQuantity,
    axis: Axis,
    timeout: Duration,
) -> anyhow::Result<()> {
    const WINDOW: f64 = 1.0;

    info!("Waiting until axis: {axis:?} target is reached: {target:?}");

    let target_print = target.clone();
    let axis_print = axis.clone();

    wait_until_event_matches(
        event_rx,
        move |event| match (event, &target) {
            (
                GantryEvent::Position {
                    axis: event_axis,
                    value,
                },
                TargetQuantity::Position(target_val),
            ) => {
                if *event_axis == axis {
                    info!("Axis: {axis:?} - checking position event value: {value} against target: {target_val}");
                    return (value - target_val).abs() <= WINDOW;
                }
                false
            }

            (
                GantryEvent::Velocity {
                    axis: event_axis,
                    value,
                },
                TargetQuantity::Velocity(target_val),
            ) if *event_axis == axis && (value - target_val).abs() <= WINDOW => true,

            (
                GantryEvent::Torque {
                    axis: event_axis,
                    value,
                },
                TargetQuantity::Torque(target_val),
            ) if *event_axis == axis && (value - target_val).abs() <= WINDOW => true,

            (
                GantryEvent::Homing {
                    axis: event_axis,
                    completed,
                    error,
                    ..
                },
                TargetQuantity::Home(reached),
            ) => *event_axis == axis && completed == reached && !(*(error)),

            _ => false,
        },
        timeout,
        format!("Target: {:?} - Axis: {:?}", target_print, axis_print),
    )
    .await
}

/// Waits until a given event [`E`] matches a predicate [`F`]
pub async fn wait_until_event_matches<F, E>(
    mut event_rx: broadcast::Receiver<E>,
    predicate: F,
    timeout: Duration,
    context: String,
) -> anyhow::Result<()>
where
    F: Fn(&E) -> bool,
    E: Clone,
{
    let deadline = Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            warn!("Timeout while waiting for event: {context}");
            bail!("Timeout when waiting for event: {context}");
        }

        let result = time::timeout(remaining, event_rx.recv()).await;

        match result {
            Ok(Ok(event)) => {
                if predicate(&event) {
                    return Ok(());
                }
            }
            Ok(Err(err @ broadcast::error::RecvError::Lagged(_))) => {
                error!("Lagged in wait_for_event, indicates serious issue");
                bail!("Lagged in wait_for_event, indicates serious issue: {err}");
            }
            Ok(Err(err @ broadcast::error::RecvError::Closed)) => {
                error!("Event channel closed in wait_for_event");
                bail!("Event channel closed in wait_for_event - {err}");
            }
            Err(_) => {
                warn!("Timeout while waiting for event: {context}");
                bail!("Timeout when waiting for event: {context}");
            }
        }
    }
}

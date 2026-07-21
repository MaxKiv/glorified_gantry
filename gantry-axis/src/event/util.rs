use anyhow::bail;
use tokio::{
    sync::broadcast,
    time::{self, Duration, Instant},
};
use tracing::*;
use futures::future::{join_all, try_join_all, TryFutureExt};
use uom::si::{f32::Velocity, length::millimeter, torque::newton_meter, velocity::meter_per_second};

use crate::{axis::{setpoint::{AxisSetpoint, PositionSetpoint, TorqueSetpoint, VelocitySetpoint}, Axis, AxisConfig}, cfg::GantryConfig, command::GantryCommand, event::GantryEvent, gantry::Gantry};

pub const TIMEOUT: Duration = Duration::from_secs(60);
pub const HOME_TIMEOUT: Duration = Duration::from_secs(60);

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
    const POS_WINDOW: f64 = 0.01;
    const VEL_WINDOW: f64 = 0.01;
    const TORQUE_WINDOW: f64 = 0.01;

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
                    return (value - target_val).abs() <= POS_WINDOW;
                }
                false
            }

            (
                GantryEvent::Velocity {
                    axis: event_axis,
                    value,
                },
                TargetQuantity::Velocity(target_val),
            ) if *event_axis == axis && (value - target_val).abs() <= VEL_WINDOW => true,

            (
                GantryEvent::Torque {
                    axis: event_axis,
                    value,
                },
                TargetQuantity::Torque(target_val),
            ) => {
                if *event_axis == axis { 
                    info!("Axis: {axis:?} - checking torque event value: {value} against target: {target_val}");
                    (value - target_val).abs() <= TORQUE_WINDOW
                } else {
                    false
                }
            }

            // (
            //     GantryEvent::PositionModeFeedback {
            //         axis: event_axis,
            //         target_reached,
            //         ..
            //     },
            //     TargetQuantity::Position(target_val),
            // ) => {
            //     if *event_axis == axis {
            //         info!("Axis: {axis:?} - checking PositionModeFeedback event against target: {target_val}");
            //         return *target_reached
            //     }
            //     false
            // }
            // (
            //     GantryEvent::TorqueModeFeedback {
            //         axis: event_axis,
            //         setpoint_reached,
            //         ..
            //     },
            //     TargetQuantity::Torque(target_val),
            // ) => {
            //     if *event_axis == axis { 
            //         info!("Axis: {axis:?} - checking TorqueModeFeedback event against target: {target_val}");
            //         *setpoint_reached
            //     } else {
            //         false
            //     }
            // }

            (
                GantryEvent::Homing {
                    axis: event_axis,
                    completed,
                    error,
                    ..
                },
                TargetQuantity::Home(reached),
            ) => *event_axis == axis && completed == reached && !(*(error)),

            // NOTE: defaults to false, meaning any other event type is irrelevant to the current target
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

pub async fn wait_for_position_target_reached(
    event_rx: broadcast::Receiver<GantryEvent>,
    timeout: Duration,
) -> anyhow::Result<()> {
    wait_until_event_matches(
        event_rx,
        |event| {
            matches!(
                event,
                GantryEvent::PositionModeFeedback {
                    target_reached: true,
                    ..
                }
            )
        },
        timeout,
        String::from("PositionModeFeedback::target_reached"),
    )
    .await
}

pub async fn wait_until_gantry_homed(
    event_rx: broadcast::Receiver<GantryEvent>,
    gantry: &Gantry,
    timeout: Duration,
) -> anyhow::Result<()> {
    wait_until_gantry_command_completed(GantryCommand::Home, event_rx, gantry, &gantry.cfg, timeout).await?;

    Ok(())
}

pub async fn wait_until_gantry_command_completed(
    cmd: GantryCommand,
    event_rx: broadcast::Receiver<GantryEvent>,
    gantry: &Gantry,
    cfg: &GantryConfig,
    timeout: Duration,
) -> anyhow::Result<()> {
    // Determine per-axis target quantity based on the given command
    let target_quantity = match &cmd {
        GantryCommand::Home => Some(TargetQuantity::Home(true)),

        GantryCommand::Setpoint { x, y, z } => {
            // Pick the first non-None axis to infer target quantity
            let first = x.as_ref().or(y.as_ref()).or(z.as_ref());
            first.map(axis_setpoint_to_target_quantity)
        }
    };

    // Build futures if target quantity is defined
    let futures = match target_quantity {
        Some(target) => vec![
            cfg.x.as_ref().map(|x| {
                wait_for_target_reached(
                    event_rx.resubscribe(),
                    target.clone(),
                    x.axis.clone(),
                    timeout,
                )
            }),
            cfg.y.as_ref().map(|y| {
                wait_for_target_reached(
                    event_rx.resubscribe(),
                    target.clone(),
                    y.axis.clone(),
                    timeout,
                )
            }),
            cfg.z.as_ref().map(|z| {
                wait_for_target_reached(
                    event_rx.resubscribe(),
                    target.clone(),
                    z.axis.clone(),
                    timeout,
                )
            }),
        ],
        _ => vec![], // no target (shouldn't happen for valid commands)
    };

    info!("xxx Sending gantry command: {cmd:?}");
    gantry.send_command(cmd.clone()).await?;
    info!("xxx Waiting until command: {cmd:?} is completed");

    let futures = futures.into_iter().flatten();
    join_all(futures).await;

    info!("xxx Gantry command completed: {cmd:?}");

    Ok(())
}


fn axis_setpoint_to_target_quantity(sp: &AxisSetpoint) -> TargetQuantity {
    match sp {
        AxisSetpoint::RelativePosition(PositionSetpoint {
            target,
        ..}) => {
            TargetQuantity::Position(target.get::<millimeter>())
        },
       AxisSetpoint::AbsolutePosition(PositionSetpoint {
            target,
        ..}) => {
            TargetQuantity::Position(target.get::<millimeter>())
        },
        AxisSetpoint::Velocity(VelocitySetpoint{target}) =>
        TargetQuantity::Velocity(target.get::<meter_per_second>()),
        AxisSetpoint::Torque(TorqueSetpoint{target}) =>
        TargetQuantity::Torque(target.get::<newton_meter>()),
    }
}


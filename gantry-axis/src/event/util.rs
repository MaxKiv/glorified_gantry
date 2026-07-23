use anyhow::bail;
use futures::future::join_all;
use oze_canopen::canopen::NodeId;
use tokio::{
    sync::broadcast,
    time::{self, Duration, Instant},
};
use tracing::*;
use uom::si::{length::millimeter, torque::newton_meter, velocity::meter_per_second};

use crate::{
    axis::{
        Axis,
        setpoint::{AxisSetpoint, PositionSetpoint, TorqueSetpoint, VelocitySetpoint},
    },
    command::GantryCommand,
    event::{GantryMotorEvent, GantryMotorEventContent},
    gantry::Gantry,
};

pub const TIMEOUT: Duration = Duration::from_secs(60);
pub const HOME_TIMEOUT: Duration = Duration::from_secs(120);

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
    event_rx: broadcast::Receiver<GantryMotorEvent>,
    target: TargetQuantity,
    node: NodeId,
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
        move |event| {
            event.axis == axis // Is this event for the right axis?
                && event.motor == node // Is this event for the right motor?
                && match (event.content, &target) { // Does this event indicate target quantity is reached?
                    (
                        GantryMotorEventContent::Position { value },
                        TargetQuantity::Position(target_val),
                    ) => (value - target_val).abs() <= POS_WINDOW,

                    (
                        GantryMotorEventContent::Velocity { value },
                        TargetQuantity::Velocity(target_val),
                    ) => (value - target_val).abs() <= VEL_WINDOW,

                    (
                        GantryMotorEventContent::Torque { value },
                        TargetQuantity::Torque(target_val),
                    ) => (value - target_val).abs() <= TORQUE_WINDOW,

                    (
                        GantryMotorEventContent::Homing {
                            completed, error, ..
                        },
                        TargetQuantity::Home(reached),
                    ) => completed == *reached && !(error),

                    // NOTE: defaults to false, meaning any other event type is irrelevant to the current target
                    _ => false,
                }
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

pub async fn send_cmd_and_wait_until_gantry_command_completed(
    cmd: GantryCommand,
    event_rx: broadcast::Receiver<GantryMotorEvent>,
    gantry: &Gantry,
    timeout: Duration,
) -> anyhow::Result<()> {
    info!("xxx Sending gantry command: {cmd:?}");
    gantry.send_command(cmd.clone()).await?;

    info!("xxx Waiting until command: {cmd:?} is completed");
    wait_until_cmd_completed(cmd.clone(), event_rx, gantry, timeout).await?;

    info!("xxx Gantry command completed: {cmd:?}");
    Ok(())
}

pub async fn wait_until_cmd_completed(
    cmd: GantryCommand,
    event_rx: broadcast::Receiver<GantryMotorEvent>,
    gantry: &Gantry,
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

    let cfg = &gantry.cfg;

    // Build master and slave futures if target quantity is defined
    let mut futures = Vec::with_capacity(6);
    if let Some(target) = &target_quantity {
        if let Some(cfg) = &cfg.x {
            futures.push(Some(wait_for_target_reached(
                event_rx.resubscribe(),
                target.clone(),
                cfg.master.node_id,
                cfg.axis.clone(),
                timeout,
            )));

            if let Some(slave) = &cfg.slave {
                futures.push(Some(wait_for_target_reached(
                    event_rx.resubscribe(),
                    target.clone(),
                    slave.node_id,
                    cfg.axis.clone(),
                    timeout,
                )));
            }
        };
        if let Some(cfg) = &cfg.y {
            futures.push(Some(wait_for_target_reached(
                event_rx.resubscribe(),
                target.clone(),
                cfg.master.node_id,
                cfg.axis.clone(),
                timeout,
            )));

            if let Some(slave) = &cfg.slave {
                futures.push(Some(wait_for_target_reached(
                    event_rx.resubscribe(),
                    target.clone(),
                    slave.node_id,
                    cfg.axis.clone(),
                    timeout,
                )));
            }
        };
        if let Some(cfg) = &cfg.z {
            futures.push(Some(wait_for_target_reached(
                event_rx.resubscribe(),
                target.clone(),
                cfg.master.node_id,
                cfg.axis.clone(),
                timeout,
            )));

            if let Some(slave) = &cfg.slave {
                futures.push(Some(wait_for_target_reached(
                    event_rx.resubscribe(),
                    target.clone(),
                    slave.node_id,
                    cfg.axis.clone(),
                    timeout,
                )));
            }
        };
    };

    // Await all futures, meaning all master and slave nodes must have their appropriate target reached
    join_all(futures.into_iter().flatten()).await;

    Ok(())
}

fn axis_setpoint_to_target_quantity(sp: &AxisSetpoint) -> TargetQuantity {
    match sp {
        AxisSetpoint::RelativePosition(PositionSetpoint { target, .. }) => {
            TargetQuantity::Position(target.get::<millimeter>())
        }
        AxisSetpoint::AbsolutePosition(PositionSetpoint { target, .. }) => {
            TargetQuantity::Position(target.get::<millimeter>())
        }
        AxisSetpoint::Velocity(VelocitySetpoint { target }) => {
            TargetQuantity::Velocity(target.get::<meter_per_second>())
        }
        AxisSetpoint::Torque(TorqueSetpoint { target }) => {
            TargetQuantity::Torque(target.get::<newton_meter>())
        }
    }
}

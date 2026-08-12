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

/// Waits until a target is reached for the given axis
/// Note: The accuracy window
pub async fn wait_for_axis_setpoint_complete(
    event_rx: broadcast::Receiver<GantryMotorEvent>,
    sp: AxisSetpoint,
    node: NodeId,
    axis: Axis,
    timeout: Duration,
) -> anyhow::Result<()> {
    const POS_WINDOW: f64 = 0.01;
    const VEL_WINDOW: f64 = 0.01;
    const TORQUE_WINDOW: f64 = 0.01;

    info!("Waiting until axis: {axis:?} target is reached: {sp:?}");

    let target_print = sp.clone();
    let axis_print = axis.clone();

    wait_until_event_matches(
        event_rx,
        move |event| {
            event.axis == axis // Is this event for the right axis?
                && event.motor == node // Is this event for the right motor?
                && match (event.content, &sp) { // Does this event indicate target quantity is reached?
                    // For absolute position mode check if current position is within target
                    (
                        GantryMotorEventContent::Position { value },
                        AxisSetpoint::AbsolutePosition(PositionSetpoint { target, velocity: _ })
                    ) => (value - target.get::<millimeter>()).abs() <= POS_WINDOW,

                    // TODO: Relative Position moves still wait for motor drives to acknowledge setpoint
                    (
                        GantryMotorEventContent::PositionModeFeedback { target_reached,
                        .. },
                        AxisSetpoint::RelativePosition(_)
                    ) => target_reached,

                    (
                        GantryMotorEventContent::Velocity { value },
                        AxisSetpoint::Velocity(VelocitySetpoint {target})
                    ) => (value - target.get::<meter_per_second>()).abs() <= VEL_WINDOW,

                    (
                        GantryMotorEventContent::Torque { value },
                        AxisSetpoint::Torque(TorqueSetpoint { target })
                    ) => (value - target.get::<newton_meter>()).abs() <= TORQUE_WINDOW,

                    // NOTE: defaults to false, meaning any other event type is irrelevant to the current target
                    _ => false,
                }
        },
        timeout,
        format!("Target: {:?} - Axis: {:?}", target_print, axis_print),
    )
    .await
}

pub async fn wait_for_axis_homed(
    event_rx: broadcast::Receiver<GantryMotorEvent>,
    node: NodeId,
    axis: Axis,
    timeout: Duration,
) -> anyhow::Result<()> {
    const POS_WINDOW: f64 = 0.01;
    const VEL_WINDOW: f64 = 0.01;
    const TORQUE_WINDOW: f64 = 0.01;

    info!("Waiting until axis: {axis:?} is homed");

    let axis_print = axis.clone();

    wait_until_event_matches(
        event_rx,
        move |event| {
            event.axis == axis // Is this event for the right axis?
                && event.motor == node // Is this event for the right motor?
                && match event.content { // Does this event indicate target quantity is reached?
                    GantryMotorEventContent::Homing { completed, error, ..
                    } => completed && !(error),

                    // NOTE: defaults to false, meaning any other event type is irrelevant to the current target
                    _ => false,
                }
        },
        timeout,
        format!("Axis: {:?} - Wait for Homed", axis_print),
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
    info!("Sending gantry command: {cmd:?}");
    gantry.send_command(cmd.clone()).await?;

    info!("Waiting until command: {cmd:?} is completed");
    wait_until_cmd_completed(cmd.clone(), event_rx, gantry, timeout).await?;

    info!("Gantry command completed: {cmd:?}");
    Ok(())
}

pub async fn wait_until_cmd_completed(
    cmd: GantryCommand,
    event_rx: broadcast::Receiver<GantryMotorEvent>,
    gantry: &Gantry,
    timeout: Duration,
) -> anyhow::Result<()> {
    // Build master and slave futures if target quantity is defined
    // Initialize to 6x None
    match cmd {
        // Build future set that waits for each axis (master + optional slave) setpoint target reached
        GantryCommand::Setpoint { x, y, z } => {
            let mut futures = Vec::new();

            for (cfg, sp) in [&gantry.cfg.x, &gantry.cfg.y, &gantry.cfg.z]
                .iter()
                .zip([x, y, z])
            {
                if let Some(cfg) = cfg
                    && let Some(sp) = sp
                {
                    futures.push(Some(wait_for_axis_setpoint_complete(
                        event_rx.resubscribe(),
                        sp.clone(),
                        cfg.master.node_id,
                        cfg.axis.clone(),
                        timeout,
                    )));
                    if let Some(slave) = &cfg.slave {
                        futures.push(Some(wait_for_axis_setpoint_complete(
                            event_rx.resubscribe(),
                            sp,
                            slave.node_id,
                            cfg.axis.clone(),
                            timeout,
                        )));
                    }
                }
            }

            // Await all futures, meaning all master and slave nodes must have their appropriate target reached
            let x = join_all(futures.into_iter().flatten()).await;
            for r in x {
                if r.is_err() {
                    return r;
                }
            }
        }

        // Build future set that waits for each axis to be homed
        GantryCommand::Home => {
            let mut futures = Vec::new();
            for cfg in [&gantry.cfg.x, &gantry.cfg.y, &gantry.cfg.z] {
                if let Some(cfg) = cfg {
                    futures.push(Some(wait_for_axis_homed(
                        event_rx.resubscribe(),
                        cfg.master.node_id,
                        cfg.axis.clone(),
                        timeout,
                    )));
                    if let Some(slave) = &cfg.slave {
                        futures.push(Some(wait_for_axis_homed(
                            event_rx.resubscribe(),
                            slave.node_id,
                            cfg.axis.clone(),
                            timeout,
                        )));
                    }
                }
            }

            // Await all futures, meaning all master and slave nodes must have their appropriate target reached
            let x = join_all(futures.into_iter().flatten()).await;
            for r in x {
                if r.is_err() {
                    return r;
                }
            }
        }
    };

    Ok(())
}

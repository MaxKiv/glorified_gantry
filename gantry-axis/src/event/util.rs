use anyhow::bail;
use tokio::{
    sync::broadcast,
    time::{self, Duration, Instant},
};
use tracing::*;

use crate::{axis::{setpoint::{AxisSetpoint, PositionSetpoint}, Axis, AxisConfig}, cfg::GantryConfig, command::GantryCommand, event::GantryEvent, gantry::Gantry};

pub const TIMEOUT: Duration = Duration::from_secs(60);
pub const HOME_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub enum TargetQuantity {
    Home(bool),
    Position(f64),
    Velocity(f64),
    Torque(f64),
}

impl TargetQuantity {
    pub fn try_from_cmd(cmd: GantryCommand) -> Option<Self> {
        match cmd {
            GantryCommand::Setpoint => {
                // Dirty hack, forgive me im tired
                cmd.map_axes(|axis| {match axis {
                    AxisSetpoint::RelativePosition(PositionSetpoint{target, ..}) =>
                    Some(TargetQuantity::Position(target)),
                }})

            }
            GantryCommand::Home => Some(Self::Home(true)),
        }
    }
}

/// Waits until a target is reached for the given axis
/// Note: The accuracy window
pub async fn wait_for_target_reached(
    event_rx: broadcast::Receiver<GantryEvent>,
    target: TargetQuantity,
    axis: Axis,
    timeout: Duration,
) -> anyhow::Result<()> {
    const POS_WINDOW: f64 = 1.0;
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
                GantryEvent::PositionModeFeedback {
                    axis: event_axis,
                    target_reached,
                    ..
                },
                TargetQuantity::Position(target_val),
            ) => {
                if *event_axis == axis {
                    info!("Axis: {axis:?} - checking PositionModeFeedback event against target: {target_val}");
                    return *target_reached
                }
                false
            }


            (
                GantryEvent::Velocity {
                    axis: event_axis,
                    value,
                },
                TargetQuantity::Velocity(target_val),
            ) if *event_axis == axis && (value - target_val).abs() <= POS_WINDOW => true,

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

            (
                GantryEvent::TorqueModeFeedback {
                    axis: event_axis,
                    setpoint_reached,
                    ..
                },
                TargetQuantity::Torque(target_val),
            ) => {
                if *event_axis == axis { 
                    info!("Axis: {axis:?} - checking TorqueModeFeedback event against target: {target_val}");
                    *setpoint_reached
                } else {
                    false
                }
            }

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

pub async fn send_commmand_and_wait_until_completed(
    cmd: GantryCommand,
    event_rx: broadcast::Receiver<GantryEvent>,
    gantry: &Gantry,
    cfg: &GantryConfig,
    timeout: Duration,
) -> anyhow::Result<()> {

    // Transform 
    let target = match cmd {
        GantryCommand::Setpoint { x, y, z } => {

        }
        GantryCommand::Home => TargetQuantity::Home(true),
    }



    let fut_x = if let Some(x) = &cfg.x {
        wait_for_target_reached(event_rx, target, x.axis, timeout)
    } else {
        std::future::ready(Some(()));
    };

    info!("Sending gantry command: {cmd:?}");
    gantry.send_command(cmd).await?;

    info!("Waiting until command: {cmd:?} is completed");

    tokio::try_join!(
        wait_for_target_reached(
            gantry.get_event_rx(),
            TargetQuantity::Home(true),
            Axis::X,
            HOME_TIMEOUT,
        ),
        wait_for_target_reached(
            gantry.get_event_rx(),
            TargetQuantity::Home(true),
            Axis::Y,
            HOME_TIMEOUT,
        ),
        wait_for_target_reached(
            gantry.get_event_rx(),
            TargetQuantity::Home(true),
            Axis::Z,
            HOME_TIMEOUT,
        ),
    )?;

    info!("TEST: Gantry homed!");

    Ok(())
}

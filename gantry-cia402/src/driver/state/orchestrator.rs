use std::time::Duration;

use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
    time::timeout,
};
use tracing::*;

use crate::{
    driver::{command::MotorCommand, state::Cia402State},
    error::DriveError,
};

const CIA402_TRANSITION_TIMEOUT: Duration = Duration::from_millis(1000);

pub async fn cia402_orchestrator_task(
    sm_cmd_tx: mpsc::Sender<Cia402State>,
    mut sm_state_rx: broadcast::Receiver<Cia402State>,
    mut cmd_rx: broadcast::Receiver<MotorCommand>,
) {
    trace!("Orchestrator task started; waiting for initial state from SM task");

    // We keep track of the current cia402 State machine state
    let mut current_state = loop {
        if let Ok(state) = sm_state_rx.recv().await {
            break state;
        }
    };
    trace!(
        "Cia402 Orchestrator received initial state from SM task: {:?} - Starting main cia402 orchestrator routine",
        current_state
    );

    // Track the current target state, this is changed by the user through cmd_rx
    let mut target_state: Option<Cia402State> = None;

    // Keep track of the currently running state transition, so we can cancel it when the current_state gets stale
    let mut current_task: Option<JoinHandle<Result<(), DriveError>>> = None;

    loop {
        tokio::select! {
            Ok(new_state) = sm_state_rx.recv() => {
                trace!("Orchestrator received SM state update: {:?}", new_state);
                current_state = new_state;

                // Cancel any ongoing transition attempt
                if let Some(task) = current_task.take() {
                    task.abort();
                    trace!("Aborted current transition due to new state update");
                }

                // If we still have a target, recalculate transition path and restart transition
                if let Some(target) = target_state {
                    if current_state != target {
                        trace!("Restarting transition from {current_state:?} toward {target:?} after state update");
                        current_task = Some(spawn_transition_task(
                            target,
                            current_state,
                            sm_cmd_tx.clone(),
                            sm_state_rx.resubscribe(),
                        ));
                    } else {
                        trace!("Target {:?} reached after state update", target);
                        target_state = None;
                    }
                }
            }

            Ok(cmd) = cmd_rx.recv() => {
                trace!("Orchestrator received command: {:?}", cmd);

                // Cancel current task
                if let Some(task) = current_task.take() {
                    task.abort();
                    trace!("Aborted current transition due to new command");
                }

                // Determine new target state
                let new_target = match cmd {
                    MotorCommand::Enable => Cia402State::OperationEnabled,
                    MotorCommand::Disable => Cia402State::ReadyToSwitchOn,
                    MotorCommand::Cia402TransitionTo { target_state } => target_state,
                    _ => continue,
                };
                target_state = Some(new_target);

                // Start fresh transition attempt
                trace!("Starting transition toward {:?}", new_target);
                current_task = Some(spawn_transition_task(
                    new_target,
                    current_state,
                    sm_cmd_tx.clone(),
                    sm_state_rx.resubscribe(),
                ));
            }

            else => {
                error!("Orchestrator: Both command and SM state feedback channels are closed, this should never happen");
            }
        }
    }
}

fn spawn_transition_task(
    to: Cia402State,
    from: Cia402State,
    sm_cmd_tx: mpsc::Sender<Cia402State>,
    mut state_rx: broadcast::Receiver<Cia402State>,
) -> JoinHandle<Result<(), DriveError>> {
    tokio::spawn(async move {
        let path = calculate_transition_path(&from, &to)
            .ok_or(DriveError::Cia402TransitionError(from, to))?;

        if path.is_empty() {
            info!(
                "requested transition from {from:?} to {to:?}, Orchestrator is already in this state"
            );
            return Ok(());
        } else {
            info!("requested transition from {from:?} to {to:?} => path: {path:?}");
        }

        for state in path.iter() {
            loop {
                info!(
                    "Requesting transition to state {state:?}, part of path: {:?}",
                    path,
                );
                sm_cmd_tx
                    .send(*state)
                    .await
                    .map_err(DriveError::Cia402SendError)?;

                // Wait for state change confirmation (with timeout)
                match timeout(CIA402_TRANSITION_TIMEOUT, state_rx.recv()).await {
                    Ok(Ok(new_state)) => {
                        trace!("orchestrator received new state from SM: {new_state:?}");
                        // Got an event within the timeout
                        if new_state == to {
                            trace!("Reached target state: {:?}", new_state);
                            return Ok(());
                        }

                        if new_state == *state {
                            trace!("Reached next state in path: {:?}", new_state);
                            break;
                        }
                    }
                    _ => {
                        // Timeout expired
                        warn!(
                            "Timeout waiting for state transition to {:?}, part of transition path {path:?}",
                            to
                        );
                        return Err(DriveError::Cia402TransitionTimeout(from, to));
                    }
                }
            }
        }

        Ok(())
    })
}

/// Defines paths through the cia402 state machine from current state to target
fn calculate_transition_path(from: &Cia402State, to: &Cia402State) -> Option<Vec<Cia402State>> {
    use Cia402State::*;

    match (from, to) {
        (s, t) if s == t => Some(vec![]),
        (NotReadyToSwitchOn, SwitchOnDisabled) => Some(vec![]), // The device should handle this automagically

        (SwitchOnDisabled, ReadyToSwitchOn) => Some(vec![ReadyToSwitchOn]),
        (SwitchOnDisabled, SwitchedOn) => Some(vec![ReadyToSwitchOn, SwitchedOn]),
        (SwitchOnDisabled, OperationEnabled) => {
            Some(vec![ReadyToSwitchOn, SwitchedOn, OperationEnabled])
        }

        (ReadyToSwitchOn, OperationEnabled) => Some(vec![SwitchedOn, OperationEnabled]),
        (ReadyToSwitchOn, SwitchedOn) => Some(vec![SwitchedOn]),
        (ReadyToSwitchOn, SwitchOnDisabled) => Some(vec![SwitchOnDisabled]),

        (SwitchedOn, SwitchOnDisabled) => Some(vec![ReadyToSwitchOn, SwitchOnDisabled]),
        (SwitchedOn, ReadyToSwitchOn) => Some(vec![ReadyToSwitchOn]),
        (SwitchedOn, OperationEnabled) => Some(vec![OperationEnabled]),

        (OperationEnabled, SwitchedOn) => Some(vec![SwitchedOn]),
        (OperationEnabled, ReadyToSwitchOn) => Some(vec![SwitchedOn, ReadyToSwitchOn]),
        (OperationEnabled, SwitchOnDisabled) => {
            Some(vec![SwitchedOn, ReadyToSwitchOn, SwitchOnDisabled])
        }

        (Fault, _) => Some(vec![SwitchOnDisabled]),
        (QuickStopActive, _) => Some(vec![SwitchOnDisabled]),
        _ => None,
    }
}

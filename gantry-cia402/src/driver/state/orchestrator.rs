use std::time::Duration;

use tokio::{
    sync::{broadcast, mpsc},
    time::timeout,
};
use tracing::*;

use crate::{
    driver::{command::MotorCommand, state::Cia402State},
    error::DriveError,
};

const CIA402_TRANSITION_TIMEOUT: Duration = Duration::from_millis(2500);

pub async fn cia402_orchestrator_task(
    mut sm_cmd_tx: mpsc::Sender<Cia402State>,
    mut sm_state_rx: mpsc::Receiver<Cia402State>,
    mut cmd_rx: broadcast::Receiver<MotorCommand>,
) {
    trace!("Orchestrator task started; waiting for initial state from SM task");

    // We keep track of the current cia402 State machine state
    let mut current_state = loop {
        if let Some(state) = sm_state_rx.recv().await {
            break state;
        }
    };

    trace!(
        "Cia402 Orchestrator received initial state from SM task: {:?} - Starting main cia402 orchestrator routine",
        current_state
    );

    // Subscribe to updates from the state machine
    loop {
        tokio::select! {
            Ok(cmd) = cmd_rx.recv() => {
                trace!("Orchestrator Received Command: {cmd:?}");
                match cmd {
                    MotorCommand::Cia402TransitionTo{target_state} => {
                        trace!("Orchestrator received request to transition from {:?} → {:?}", current_state, target_state);
                        if let Err(e) = transition_to_state(target_state, &mut current_state, &mut sm_cmd_tx, &mut sm_state_rx).await {
                            error!("Transition failed: {e}");
                        }
                    }
                    MotorCommand::Enable => {
                        trace!("{cmd:?} -> Transitioning to Cia402State::OperationEnabled");
                        if let Err(e) = transition_to_state(Cia402State::OperationEnabled, &mut
                        current_state, &mut sm_cmd_tx, &mut sm_state_rx).await {
                            error!("Transition failed: {e}");
                        }
                    }
                    MotorCommand::Disable => {
                        trace!("{cmd:?} -> Transitioning to Cia402State::SwitchOnDisabled");
                        if let Err(e) = transition_to_state(Cia402State::SwitchOnDisabled, &mut
                        current_state, &mut sm_cmd_tx, &mut sm_state_rx).await {
                            error!("Transition failed: {e}");
                        }
                    }
                    MotorCommand::ResetFault => {
                        trace!("{cmd:?} -> Transitioning to Cia402State::SwitchOnDisabled");
                        if let Err(e) = transition_to_state(Cia402State::SwitchOnDisabled, &mut
                        current_state, &mut sm_cmd_tx, &mut sm_state_rx).await {
                            error!("Transition failed: {e}");
                        }
                    }
                    _ => {}
                }
            }

            Some(state) = sm_state_rx.recv() => {
                trace!("Orchestrator received state update from cia402 SM: {:?}", state);
                current_state = state;
            }
        }
    }
}

async fn transition_to_state(
    to: Cia402State,
    from: &mut Cia402State,
    sm_cmd_tx: &mut mpsc::Sender<Cia402State>,
    state_rx: &mut mpsc::Receiver<Cia402State>,
) -> Result<(), DriveError> {
    let path = get_path(from, &to).ok_or(DriveError::Cia402TransitionError(*from, to))?;

    if path.is_empty() {
        info!(
            "requested transition from {from:?} to {to:?}, Orchestrator is already in this state"
        );
        return Ok(());
    } else {
        info!("requested transition from {from:?} to {to:?} => path: {path:?}");
    }

    for state in path.iter() {
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
            Ok(Some(new_state)) => {
                trace!("orchestrator received new state from SM: {new_state:?}");
                // Got an event within the timeout
                if new_state == *state {
                    trace!("Reached target state: {:?}", new_state);
                    return Ok(());
                }
            }
            _ => {
                // Timeout expired
                warn!(
                    "Timeout waiting for state transition to {:?}, part of transition path {path:?}",
                    to
                );
                return Err(DriveError::Cia402TransitionTimeout(*from, to));
            }
        }
    }

    Ok(())
}

/// Defines paths through the cia402 state machine from current state to target
fn get_path(from: &Cia402State, to: &Cia402State) -> Option<Vec<Cia402State>> {
    use Cia402State::*;

    match (from, to) {
        (s, t) if s == t => Some(vec![]),
        (SwitchOnDisabled, OperationEnabled) => {
            Some(vec![ReadyToSwitchOn, SwitchedOn, OperationEnabled])
        }
        (OperationEnabled, SwitchOnDisabled) => {
            Some(vec![SwitchedOn, ReadyToSwitchOn, SwitchOnDisabled])
        }
        (ReadyToSwitchOn, SwitchedOn) => Some(vec![SwitchedOn]),
        (SwitchedOn, ReadyToSwitchOn) => Some(vec![ReadyToSwitchOn]),
        (SwitchOnDisabled, ReadyToSwitchOn) => Some(vec![ReadyToSwitchOn]),
        (ReadyToSwitchOn, SwitchOnDisabled) => Some(vec![SwitchOnDisabled]),
        (Fault, _) => Some(vec![SwitchOnDisabled]),
        (QuickStopActive, _) => Some(vec![SwitchOnDisabled]),
        _ => None,
    }
}

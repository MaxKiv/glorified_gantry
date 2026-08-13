use tokio::sync::{broadcast, mpsc};
use tracing::*;

use crate::{
    driver::{
        event::MotorEvent,
        state::{Cia402Flags, Cia402State},
    },
    error::DriveError,
};

pub enum Cia402Command {
    Update(Cia402Flags),
    Transition(Cia402Flags),
}

pub struct Cia402StateMachine {
    pub state: Cia402State,
}

pub async fn cia402_state_machine_task(
    mut event_rx: broadcast::Receiver<MotorEvent>,
    update_tx: mpsc::Sender<Cia402Command>,
    sm_state_tx: broadcast::Sender<Cia402State>,
    mut sm_cmd_rx: mpsc::Receiver<Cia402State>,
    event_tx: broadcast::Sender<MotorEvent>,
) -> Result<(), DriveError> {
    trace!("Cia402 SM task started - waiting on initial state");

    let mut sm = Cia402StateMachine {
        state: loop {
            if let Ok(MotorEvent::StatusWord(sw)) = event_rx.recv().await {
                trace!("Cia402 SM received initial state update event: {sw:?}");

                // Parse it into a Cia402State
                if let Ok(state) = sw.try_into() {
                    trace!(
                        "Cia402 SM parsed initial state update event: {sw:?} into Cia402State: {state:?}"
                    );

                    // Notify the cia402 orchestrator
                    if let Err(err) = sm_state_tx.send(state) {
                        error!("Unable to send cia402 state update event: {err}");
                    } else {
                        trace!("cia402 SM send state update to orchestrator: {state:?}")
                    }

                    // Bonus: Notify event loop of the new Cia402 state
                    // This is not strictly required, but nice for [`log::log_events`]
                    if let Err(err) = event_tx.send(MotorEvent::Cia402StateUpdate(state)) {
                        error!("Unable to send cia402 state update event: {err}");
                    }

                    // Initial state received, continue with main routine
                    break state;
                }
            }
        },
    };

    trace!(
        "Cia402 SM received initial state from device: {:?} - Starting main cia402 state machine routine",
        sm.state
    );

    loop {
        tokio::select! {
            Some(cmd) = sm_cmd_rx.recv() => {
                trace!(
                    "Cia402 SM command received - cmd: {:?} - current state: {:?}",
                    cmd, sm.state
                );
                if let Some(transition_flags) = Cia402Flags::transition_flags(&sm.state, &cmd) {
                    trace!(
                        "Requested transition is valid - cia402Flags: {transition_flags:?}",
                    );

                    if let Err(err) = update_tx.send(Cia402Command::Transition(transition_flags)).await {
                        error!("Unable to request cia402 state transition from PDO: {err}" );
                    }
                } else {
                    warn!("CiA402 State machine disallows transition from {:?} to {cmd:?}", sm.state);
                }
            }

            Ok(event) = event_rx.recv() => {
                if let MotorEvent::StatusWord(sw) = event {
                    match sw.try_into() {
                        Ok(new_state) => {
                            info!(
                                "Cia402 decoded {sw:?} into new state: {new_state:?} - Informing subsystems",
                            );

                            // Notify the cia402 orchestrator
                            if let Err(err) = sm_state_tx.send(new_state){
                                error!(
                                    "Unable to send cia402 state update event: {err}"
                                );
                            } else {
                                trace!("cia402 SM send state update to orchestrator: {new_state:?}")
                            }

                            // Bonus: Notify event loop of the new Cia402 state
                            // This is not strictly required, but nice for [`log::log_events`]
                            if let Err(err) = event_tx.send(MotorEvent::Cia402StateUpdate(new_state)) {
                                error!(
                                    "Unable to send cia402 state update event: {err}"
                                );
                            }

                            sm.state = new_state
                        },
                        Err(err) => {
                            error!("{err}");
                        }
                    }
                }
            }
        }
    }
}

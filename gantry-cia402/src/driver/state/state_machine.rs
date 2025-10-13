use tokio::sync::{broadcast, mpsc};
use tracing::*;

use crate::driver::{
    command::MotorCommand,
    event::MotorEvent,
    state::{Cia402Flags, Cia402State},
};

pub struct Cia402StateMachine {
    pub state: Cia402State,
}

impl Cia402StateMachine {
    pub fn next_controlword(&self, cmd: &MotorCommand) -> Option<Cia402Flags> {
        use Cia402State::*;

        match (self.state, cmd) {
            (Fault, MotorCommand::ResetFault) => Some(Cia402Flags::FAULT_RESET),
            (SwitchOnDisabled, MotorCommand::Enable) => {
                Some(Cia402Flags::ENABLE_VOLTAGE | Cia402Flags::DISABLE_QUICK_STOP)
            }
            (ReadyToSwitchOn, MotorCommand::Enable) => Some(
                Cia402Flags::ENABLE_VOLTAGE
                    | Cia402Flags::DISABLE_QUICK_STOP
                    | Cia402Flags::SWITCH_ON,
            ),
            (SwitchedOn, MotorCommand::Enable) => Some(
                Cia402Flags::ENABLE_VOLTAGE
                    | Cia402Flags::DISABLE_QUICK_STOP
                    | Cia402Flags::SWITCH_ON
                    | Cia402Flags::ENABLE_OPERATION,
            ),
            (OperationEnabled, MotorCommand::Disable) => Some(Cia402Flags::DISABLE_QUICK_STOP),
            _ => None,
        }
    }
}

pub async fn cia402_state_machine_task(
    mut event_rx: broadcast::Receiver<MotorEvent>,
    state_update_tx: mpsc::Sender<Cia402Flags>,
    sm_state_tx: mpsc::Sender<Cia402State>,
    mut sm_cmd_rx: mpsc::Receiver<Cia402State>,
    mut event_tx: broadcast::Sender<MotorEvent>,
) {
    let mut sm = Cia402StateMachine {
        state: Cia402State::SwitchOnDisabled,
    };

    loop {
        tokio::select! {
            Some(cmd) = sm_cmd_rx.recv() => {
                if let Some(cw) = Cia402Flags::transition_flags(&sm.state, &cmd) {
                    trace!(
                        "Cia402 state update command received - from: {:?} - cmd: {:?} - controlword {:?}",
                        sm.state, cmd, cw
                    );
                    if let Err(err) = state_update_tx.send(cw).await {
                        error!("Error while processing command: {cmd:?} -> Unable to send state update request: {err}" );
                    }
                } else {
                    warn!("CiA402 State machine disallows transition from {:?} to {cmd:?}", sm.state);
                }
            }

            Ok(event) = event_rx.recv() => {
                if let MotorEvent::StatusWord(sw) = event {
                    match sw.try_into() {
                        Ok(new_state) => {
                            trace!(
                                "Cia402 decoded {sw:?} into new state: {new_state:?}",
                            );

                            // Notify the cia402 orchestrator
                            if let Err(err) = sm_state_tx.send(new_state).await {
                                error!(
                                    "Unable to send cia402 state update event: {err}"
                                );
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

pub async fn cia402_orchestrator() {}

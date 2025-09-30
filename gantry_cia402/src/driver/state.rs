use oze_canopen::interface::CanOpenInterface;
use tokio::{
    sync::mpsc::{self},
    task,
};
use tracing::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cia402State {
    #[default]
    NotReadyToSwitchOn,
    SwitchOnDisabled,
    ReadyToSwitchOn,
    SwitchedOn,
    OperationEnabled,
    QuickStopActive,
    FaultReactionActive,
    Fault,
}

impl Cia402State {
    pub fn allowed_transitions(self) -> &'static [Cia402State] {
        use Cia402State::*;
        match self {
            NotReadyToSwitchOn => &[SwitchOnDisabled],
            SwitchOnDisabled => &[ReadyToSwitchOn],
            ReadyToSwitchOn => &[SwitchedOn, SwitchOnDisabled],
            SwitchedOn => &[OperationEnabled, ReadyToSwitchOn],
            OperationEnabled => &[SwitchedOn, QuickStopActive, Fault],
            QuickStopActive => &[SwitchOnDisabled],
            FaultReactionActive => &[Fault],
            Fault => &[SwitchOnDisabled],
        }
    }
}

/// A minimal Cia402 Power State Machine implementation
/// Used by the Cia402Driver to check state transitions and track current device Cia402 state
pub struct Cia402StateMachine {
    node_id: u8,
    canopen: CanOpenInterface,

    current_state: Cia402State,
    state_cmd_rx: mpsc::Receiver<Cia402State>,
    state_feedback_rx: mpsc::Receiver<Cia402State>,
    state_update_tx: mpsc::Sender<Cia402State>,
}

impl Cia402StateMachine {
    /// Launch a task to manage the Cia402 StateMachine of a connected CANopen device
    pub fn init(
        node_id: u8,
        canopen: CanOpenInterface,
        state_cmd_rx: mpsc::Receiver<Cia402State>,
        state_feedback_rx: mpsc::Receiver<Cia402State>,
        state_update_tx: mpsc::Receiver<Cia402State>,
    ) -> Self {
        let mut cia402 = Self {
            node_id,
            canopen,
            current_state: Cia402State::NotReadyToSwitchOn,
            state_cmd_rx,
            state_feedback_rx,
            state_update_tx,
        };

        let cia402_handler = task::spawn(cia402.run());

        cia402_handler
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                // Check for cia402 state feedback from the device
                Some(new_state) = self.state_feedback_rx.recv() => {
                    trace!(
                        "Cia402 state update received, old -> new state: {:?} -> {new_state:?}",
                        // Update current state
                        self.current_state
                    );

                    self.current_state = new_state;
                }
                // Check for cia402 state transition requests from the user
                Some(new_state) = self.state_cmd_rx.recv() => {
                    trace!(
                        "Cia402 state update requested, old -> new state: {:?} -> {new_state:?}",
                        self.current_state
                    );

                    // Check if the requested transition is valid
                    if self.transition_is_valid(new_state) {
                        // Valid transition: notify Update task
                        self.state_update_tx.send(new_state).await;
                    } else {
                        error!(
                            "Invalid Cia402 state update requested, old -> new state: {:?} -> {new_state:?}",
                            self.current_state
                        );
                        // TODO: broadcast invalid state transition event?
                        // Err(DriveError::InvalidTransition(self.current_state, new_state))
                    }
                }
            }
        }
    }

    /// Check if transition to given state is valid
    /// If so: Indicate the update task that a state change is requested
    pub fn transition_is_valid(&mut self, new_state: Cia402State) -> bool {
        self.current_state
            .allowed_transitions()
            .contains(&new_state)
    }
}

use tokio::sync::mpsc::Sender;

use crate::error::DriveError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cia402State {
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

pub struct Cia402StateMachine {
    state: Cia402State,
    new_state_sender: Sender<Cia402State>,
}

impl Cia402StateMachine {
    pub fn new(new_state_sender: Sender<Cia402State>) -> Self {
        Self {
            state: Cia402State::SwitchOnDisabled,
            new_state_sender,
        }
    }

    pub async fn transition_to(&mut self, new_state: Cia402State) -> Result<(), DriveError> {
        if self.state.allowed_transitions().contains(&new_state) {
            self.new_state_sender.send(new_state).await;
            self.state = new_state;
            Ok(())
        } else {
            Err(DriveError::InvalidTransition(self.state, new_state))
        }
    }
}

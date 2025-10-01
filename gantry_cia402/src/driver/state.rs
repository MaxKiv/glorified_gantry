use oze_canopen::interface::CanOpenInterface;
use tokio::{
    sync::mpsc::{self},
    task::{self, JoinHandle},
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

pub struct Cia402Transition {
    from: Cia402State,
    to: Cia402State,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Cia402Flags: u16 {
        /// Bit 0: Switch on
        /// Requests transition from "Ready to Switch On" → "Switched On".
        const SWITCH_ON        = 1 << 0;

        /// Bit 1: Enable voltage
        /// Powers the drive (main contactor / power stage).
        const ENABLE_VOLTAGE   = 1 << 1;

        /// Bit 2: Quick stop
        /// 0 = initiate quick stop according to deceleration parameters.
        /// 1 = allow operation.
        const QUICK_STOP       = 1 << 2;

        /// Bit 3: Enable operation
        /// Allows motion commands when set, completing transition into "Operation Enabled".
        const ENABLE_OPERATION = 1 << 3;

        /// Bit 7: Fault reset
        /// Rising edge resets faults and attempts to return to "Switch On Disabled".
        const FAULT_RESET      = 1 << 7;
    }
}

impl Default for Cia402Flags {
    fn default() -> Self {
        Cia402Flags::empty()
    }
}

impl Cia402Flags {
    fn from_transition(Cia402Transition { from, to }: Cia402Transition) -> Self {
        match (from, to) {
            (Cia402State::NotReadyToSwitchOn, Cia402State::NotReadyToSwitchOn) => todo!(),
            (Cia402State::NotReadyToSwitchOn, Cia402State::SwitchOnDisabled) => todo!(),
            (Cia402State::NotReadyToSwitchOn, Cia402State::ReadyToSwitchOn) => todo!(),
            (Cia402State::NotReadyToSwitchOn, Cia402State::SwitchedOn) => todo!(),
            (Cia402State::NotReadyToSwitchOn, Cia402State::OperationEnabled) => todo!(),
            (Cia402State::NotReadyToSwitchOn, Cia402State::QuickStopActive) => todo!(),
            (Cia402State::NotReadyToSwitchOn, Cia402State::FaultReactionActive) => todo!(),
            (Cia402State::NotReadyToSwitchOn, Cia402State::Fault) => todo!(),
            (Cia402State::SwitchOnDisabled, Cia402State::NotReadyToSwitchOn) => todo!(),
            (Cia402State::SwitchOnDisabled, Cia402State::SwitchOnDisabled) => todo!(),
            (Cia402State::SwitchOnDisabled, Cia402State::ReadyToSwitchOn) => todo!(),
            (Cia402State::SwitchOnDisabled, Cia402State::SwitchedOn) => todo!(),
            (Cia402State::SwitchOnDisabled, Cia402State::OperationEnabled) => todo!(),
            (Cia402State::SwitchOnDisabled, Cia402State::QuickStopActive) => todo!(),
            (Cia402State::SwitchOnDisabled, Cia402State::FaultReactionActive) => todo!(),
            (Cia402State::SwitchOnDisabled, Cia402State::Fault) => todo!(),
            (Cia402State::ReadyToSwitchOn, Cia402State::NotReadyToSwitchOn) => todo!(),
            (Cia402State::ReadyToSwitchOn, Cia402State::SwitchOnDisabled) => todo!(),
            (Cia402State::ReadyToSwitchOn, Cia402State::ReadyToSwitchOn) => todo!(),
            (Cia402State::ReadyToSwitchOn, Cia402State::SwitchedOn) => todo!(),
            (Cia402State::ReadyToSwitchOn, Cia402State::OperationEnabled) => todo!(),
            (Cia402State::ReadyToSwitchOn, Cia402State::QuickStopActive) => todo!(),
            (Cia402State::ReadyToSwitchOn, Cia402State::FaultReactionActive) => todo!(),
            (Cia402State::ReadyToSwitchOn, Cia402State::Fault) => todo!(),
            (Cia402State::SwitchedOn, Cia402State::NotReadyToSwitchOn) => todo!(),
            (Cia402State::SwitchedOn, Cia402State::SwitchOnDisabled) => todo!(),
            (Cia402State::SwitchedOn, Cia402State::ReadyToSwitchOn) => todo!(),
            (Cia402State::SwitchedOn, Cia402State::SwitchedOn) => todo!(),
            (Cia402State::SwitchedOn, Cia402State::OperationEnabled) => todo!(),
            (Cia402State::SwitchedOn, Cia402State::QuickStopActive) => todo!(),
            (Cia402State::SwitchedOn, Cia402State::FaultReactionActive) => todo!(),
            (Cia402State::SwitchedOn, Cia402State::Fault) => todo!(),
            (Cia402State::OperationEnabled, Cia402State::NotReadyToSwitchOn) => todo!(),
            (Cia402State::OperationEnabled, Cia402State::SwitchOnDisabled) => todo!(),
            (Cia402State::OperationEnabled, Cia402State::ReadyToSwitchOn) => todo!(),
            (Cia402State::OperationEnabled, Cia402State::SwitchedOn) => todo!(),
            (Cia402State::OperationEnabled, Cia402State::OperationEnabled) => todo!(),
            (Cia402State::OperationEnabled, Cia402State::QuickStopActive) => todo!(),
            (Cia402State::OperationEnabled, Cia402State::FaultReactionActive) => todo!(),
            (Cia402State::OperationEnabled, Cia402State::Fault) => todo!(),
            (Cia402State::QuickStopActive, Cia402State::NotReadyToSwitchOn) => todo!(),
            (Cia402State::QuickStopActive, Cia402State::SwitchOnDisabled) => todo!(),
            (Cia402State::QuickStopActive, Cia402State::ReadyToSwitchOn) => todo!(),
            (Cia402State::QuickStopActive, Cia402State::SwitchedOn) => todo!(),
            (Cia402State::QuickStopActive, Cia402State::OperationEnabled) => todo!(),
            (Cia402State::QuickStopActive, Cia402State::QuickStopActive) => todo!(),
            (Cia402State::QuickStopActive, Cia402State::FaultReactionActive) => todo!(),
            (Cia402State::QuickStopActive, Cia402State::Fault) => todo!(),
            (Cia402State::FaultReactionActive, Cia402State::NotReadyToSwitchOn) => todo!(),
            (Cia402State::FaultReactionActive, Cia402State::SwitchOnDisabled) => todo!(),
            (Cia402State::FaultReactionActive, Cia402State::ReadyToSwitchOn) => todo!(),
            (Cia402State::FaultReactionActive, Cia402State::SwitchedOn) => todo!(),
            (Cia402State::FaultReactionActive, Cia402State::OperationEnabled) => todo!(),
            (Cia402State::FaultReactionActive, Cia402State::QuickStopActive) => todo!(),
            (Cia402State::FaultReactionActive, Cia402State::FaultReactionActive) => todo!(),
            (Cia402State::FaultReactionActive, Cia402State::Fault) => todo!(),
            (Cia402State::Fault, Cia402State::NotReadyToSwitchOn) => todo!(),
            (Cia402State::Fault, Cia402State::SwitchOnDisabled) => todo!(),
            (Cia402State::Fault, Cia402State::ReadyToSwitchOn) => todo!(),
            (Cia402State::Fault, Cia402State::SwitchedOn) => todo!(),
            (Cia402State::Fault, Cia402State::OperationEnabled) => todo!(),
            (Cia402State::Fault, Cia402State::QuickStopActive) => todo!(),
            (Cia402State::Fault, Cia402State::FaultReactionActive) => todo!(),
            (Cia402State::Fault, Cia402State::Fault) => todo!(),
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
    state_update_tx: mpsc::Sender<Cia402Transition>,
}

impl Cia402StateMachine {
    /// Launch a task to manage the Cia402 StateMachine of a connected CANopen device
    pub fn init(
        node_id: u8,
        canopen: CanOpenInterface,
        state_cmd_rx: mpsc::Receiver<Cia402State>,
        state_feedback_rx: mpsc::Receiver<Cia402State>,
        state_update_tx: mpsc::Sender<Cia402Transition>,
    ) -> JoinHandle<()> {
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

    pub async fn run(mut self) {
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

                        let x = Cia402Flags::from_transition(Cia402Transition {
                            from:self.current_state, to:
                            new_state
                        });

                        self.state_update_tx.send().await;
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

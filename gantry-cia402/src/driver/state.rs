use tokio::{
    sync::{
        broadcast::{self, Receiver},
        mpsc::{self},
    },
    task::{self, JoinHandle},
};
use tracing::*;

use crate::driver::{command::MotorCommand, event::MotorEvent, receiver::StatusWord};

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

impl From<StatusWord> for Cia402Flags {
    fn from(status: StatusWord) -> Self {
        Self::from_bits_truncate(status.bits())
    }
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
        const DISABLE_QUICK_STOP       = 1 << 2;

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

pub async fn cia402_task(
    mut event_rx: broadcast::Receiver<MotorEvent>,
    mut cmd_rx: Receiver<MotorCommand>,
    state_update_tx: mpsc::Sender<Cia402Flags>,
) {
    let mut sm = Cia402StateMachine {
        state: Cia402State::SwitchOnDisabled,
    };

    loop {
        tokio::select! {
            Ok(cmd) = cmd_rx.recv() => {
                if let Some(cw) = sm.next_controlword(&cmd) {
                    trace!(
                        "Cia402 state update requested - from: {:?} cmd: {:?} controlword {:?}",
                        sm.state, cmd, cw
                    );
                    state_update_tx.send(cw).await;
                }
            }
            Ok(event) = event_rx.recv() => {
                if let MotorEvent::Cia402StateUpdate(new_state) = event {
                    trace!(
                        "Cia402 Received State update: {new_state:?}",
                    );
                    sm.state = new_state;
                }
            }
        }
    }
}

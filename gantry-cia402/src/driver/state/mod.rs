pub mod orchestrator;
pub mod state_machine;

use tokio::sync::{
    broadcast::{self, Receiver},
    mpsc::{self},
};
use tracing::*;

use crate::{
    driver::{command::MotorCommand, event::MotorEvent, receiver::StatusWord},
    error::DriveError,
};

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

impl TryFrom<StatusWord> for Cia402State {
    type Error = DriveError;

    // Extract relevant bits according to datasheet page 47
    fn try_from(status: StatusWord) -> Result<Self, Self::Error> {
        // Mask bits 0–6
        const MASK: u16 = 0b11_1111;

        Ok(match status.bits() & MASK {
            0x0000 => Cia402State::NotReadyToSwitchOn,
            0x0040 => Cia402State::SwitchOnDisabled,
            0x0021 => Cia402State::ReadyToSwitchOn,
            0x0023 => Cia402State::SwitchedOn,
            0x0027 => Cia402State::OperationEnabled,
            0x0007 => Cia402State::QuickStopActive,
            0x000F => Cia402State::FaultReactionActive,
            0x0008 => Cia402State::Fault,
            _ => return Err(DriveError::Cia402StateDecode(status)),
        })
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

impl Cia402Flags {
    /// Return the controlword flags needed to move from one CiA402 state to another.
    pub fn transition_flags(from: &Cia402State, to: &Cia402State) -> Option<Cia402Flags> {
        use Cia402Flags as F;
        use Cia402State::*;

        match (from, to) {
            // ----- Fault handling -----
            (Fault, SwitchOnDisabled) => Some(F::FAULT_RESET),

            // ----- Enabling sequence -----
            // Switch On Disabled → Ready To Switch On
            (SwitchOnDisabled, ReadyToSwitchOn) => Some(F::ENABLE_VOLTAGE | F::DISABLE_QUICK_STOP),

            // Ready To Switch On → Switched On
            (ReadyToSwitchOn, SwitchedOn) => {
                Some(F::ENABLE_VOLTAGE | F::DISABLE_QUICK_STOP | F::SWITCH_ON)
            }

            // Switched On → Operation Enabled
            (SwitchedOn, OperationEnabled) => {
                Some(F::ENABLE_VOLTAGE | F::DISABLE_QUICK_STOP | F::SWITCH_ON | F::ENABLE_OPERATION)
            }

            // ----- Disabling sequence -----
            // Operation Enabled → Switched On (disable operation bit)
            (OperationEnabled, SwitchedOn) => {
                Some(F::ENABLE_VOLTAGE | F::DISABLE_QUICK_STOP | F::SWITCH_ON)
            }

            // Switched On → Ready To Switch On (switch off bit 0)
            (SwitchedOn, ReadyToSwitchOn) => Some(F::ENABLE_VOLTAGE | F::DISABLE_QUICK_STOP),

            // Ready To Switch On → Switch On Disabled (disable voltage)
            (ReadyToSwitchOn, SwitchOnDisabled) => Some(F::empty()),

            // ----- Quick Stop (runtime stop) -----
            // Operation Enabled → Quick Stop Active (clear bit2)
            (OperationEnabled, QuickStopActive) => {
                Some(F::ENABLE_VOLTAGE | F::SWITCH_ON | F::ENABLE_OPERATION)
            }

            // Quick Stop Active → Switch On Disabled (voltage off)
            (QuickStopActive, SwitchOnDisabled) => Some(F::empty()),

            // ----- Any other transitions are invalid -----
            _ => None,
        }
    }
}

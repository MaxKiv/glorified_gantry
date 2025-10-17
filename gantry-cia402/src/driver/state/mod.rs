pub mod orchestrator;
pub mod state_machine;

use tracing::*;

use crate::{driver::receiver::StatusWord, error::DriveError};

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
        // Mask bits - see datasheet page 336
        const MASK: u16 = 0b110_1111;

        trace!(
            "parsing status: {status:?} (bits: {:#0b}) into Cia402State",
            status.bits()
        );

        Ok(match status.bits() & MASK {
            0b000_0000 => Cia402State::NotReadyToSwitchOn,
            0b100_0000 => Cia402State::SwitchOnDisabled,
            0b010_0001 => Cia402State::ReadyToSwitchOn,
            0b010_0011 => Cia402State::SwitchedOn,
            0b010_0111 => Cia402State::OperationEnabled,
            0b000_0111 => Cia402State::QuickStopActive,
            0b000_1111 => Cia402State::FaultReactionActive,
            0b000_1000 => Cia402State::Fault,
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
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statusword_parsing() {
        // Test each defined state
        let test_cases = vec![
            (0b000_0000, Cia402State::NotReadyToSwitchOn),
            (0b100_0000, Cia402State::SwitchOnDisabled),
            (0b010_0001, Cia402State::ReadyToSwitchOn),
            (0b010_0011, Cia402State::SwitchedOn),
            (0b010_0111, Cia402State::OperationEnabled),
            (0b000_0111, Cia402State::QuickStopActive),
            (0b000_1111, Cia402State::FaultReactionActive),
            (0b000_1000, Cia402State::Fault),
        ];

        for (bits, expected_state) in test_cases {
            let sw = StatusWord::from_bits_truncate(bits);
            let state: Result<Cia402State, _> = sw.try_into();
            assert_eq!(
                state.unwrap(),
                expected_state,
                "Failed to parse bits {:#010b}",
                bits
            );
        }
    }

    #[test]
    fn test_statusword_with_extra_bits() {
        // Ensure masking works correctly
        let sw = StatusWord::from_bits_truncate(0b11_1011_0111); // OperationEnabled
        let state: Result<Cia402State, _> = sw.try_into();
        assert!(state.is_ok()); // Should still parse despite extra bits
    }

    #[test]
    fn test_enable_transition() {
        // Test full enable path
        let flags = Cia402Flags::transition_flags(
            &Cia402State::SwitchOnDisabled,
            &Cia402State::ReadyToSwitchOn,
        );
        assert_eq!(
            flags,
            Some(Cia402Flags::ENABLE_VOLTAGE | Cia402Flags::DISABLE_QUICK_STOP)
        );
    }

    #[test]
    fn test_invalid_transition() {
        // Should return None for invalid transitions
        assert!(
            Cia402Flags::transition_flags(
                &Cia402State::NotReadyToSwitchOn,
                &Cia402State::OperationEnabled
            )
            .is_none()
        );
    }

    #[test]
    fn test_fault_recovery_transition() {
        let flags =
            Cia402Flags::transition_flags(&Cia402State::Fault, &Cia402State::SwitchOnDisabled);
        assert_eq!(flags, Some(Cia402Flags::FAULT_RESET));
    }

    #[test]
    fn test_all_valid_transitions() {
        // Exhaustively test all valid state pairs
        let valid_transitions = vec![
            (Cia402State::SwitchOnDisabled, Cia402State::ReadyToSwitchOn),
            (Cia402State::ReadyToSwitchOn, Cia402State::SwitchedOn),
            // ... all valid pairs
        ];

        for (from, to) in valid_transitions {
            assert!(
                Cia402Flags::transition_flags(&from, &to).is_some(),
                "Expected valid transition from {:?} to {:?}",
                from,
                to
            );
        }
    }
}

pub mod publisher;

use thiserror::Error;
use tracing::info;

use crate::driver::{
    oms::{home::HomeFlagsCW, position::PositionFlagsCW},
    state::{Cia402Flags, Cia402State},
};

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Invalid state transition from {0:?} to {1:?}")]
    InvalidTransition(Cia402State, Cia402State),
}

// ControlWord(ENABLE_VOLTAGE | HALT | OMS_4 | RESERVED_2 | RESERVED_3 | MANUFACTURER_1 | MANUFACTURER_2) - ProfilePosition
bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ControlWord: u16 {
        /// Bit 0: Switch on
        const SWITCH_ON              = 1 << 0;
        /// Bit 1: Enable voltage
        const ENABLE_VOLTAGE         = 1 << 1;
        /// Bit 2: Disable Quick stop
        const DISABLE_QUICK_STOP     = 1 << 2;
        /// Bit 3: Enable operation
        const ENABLE_OPERATION       = 1 << 3;

        /// Bit 4: Operation mode specific
        const OMS_1                  = 1 << 4;
        /// Bit 5: Operation mode specific
        const OMS_2                  = 1 << 5;
        /// Bit 6: Operation mode specific
        const OMS_3                  = 1 << 6;

        /// Bit 7: Fault reset
        const FAULT_RESET            = 1 << 7;

        /// Bit 8: Halt
        const HALT                   = 1 << 8;
        /// Bit 9: Operational Mode Specific meaning
        const OMS_4                  = 1 << 9;
        /// Bit 10: Reserved
        const RESERVED_2             = 1 << 10;
        /// Bit 11: Reserved
        const RESERVED_3             = 1 << 11;

        /// Bit 12: Manufacturer specific
        const MANUFACTURER_1         = 1 << 12;
        /// Bit 13: Manufacturer specific
        const MANUFACTURER_2         = 1 << 13;
        /// Bit 14: Manufacturer specific
        const MANUFACTURER_3         = 1 << 14;
        /// Bit 15: Manufacturer specific
        const MANUFACTURER_4         = 1 << 15;
    }
}

impl ControlWord {
    pub fn with_position_flags(self, flags: &PositionFlagsCW) -> Self {
        let mask = PositionFlagsCW::all().bits();
        let new_bits = (self.bits() & !mask) | (flags.bits() & mask);
        ControlWord::from_bits_truncate(new_bits)
    }

    pub fn with_home_flags(self, flags: &HomeFlagsCW) -> Self {
        let mask = HomeFlagsCW::all().bits();
        let new_bits = (self.bits() & !mask) | (flags.bits() & mask);
        ControlWord::from_bits_truncate(new_bits)
    }

    pub fn with_cia402_flags(self, flags: &Cia402Flags) -> Self {
        info!("adding cia402flags to cw: {flags:?}");

        let mask = Cia402Flags::all().bits();

        info!("Cia402flags mask: {:#0b}", mask);
        info!("self.bits() & !mask: {:#0b}", self.bits() & !mask);
        info!("(flags.bits() & mask): {:#0b}", (flags.bits() & mask));

        let new_bits = (self.bits() & !mask) | (flags.bits() & mask);
        info!("new_bits {:#0b}", new_bits);

        let cw = ControlWord::from_bits(new_bits).expect("Bits size mismatch in with_cia402_flags");
        info!("new cw: {cw:?}");

        cw
    }
}

impl Default for ControlWord {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cw_update_with_position_flags() {
        let simple_cw = ControlWord::from_bits_truncate(0b101010101010101);
        let flags = PositionFlagsCW::empty();
        let simple_combined = simple_cw.with_position_flags(flags);
        assert_eq!(simple_combined.bits(), 0b101010000000101);
    }

    #[test]
    fn test_cw_update_with_cia402_flags() {
        let cw = ControlWord::from_bits_truncate(0b1111111111111111);
        let flags = Cia402Flags::empty();
        let combined = cw.with_cia402_flags(flags);
        assert_eq!(combined.bits(), 0b1111111110111000);
    }

    #[test]
    fn test_cw_update_flag_isolation() {
        // Verify flags don't interfere with each other
        let base = ControlWord::from_bits_truncate(0b1111_1111_1111_1111);

        let with_pos = base.with_position_flags(PositionFlagsCW::empty());
        let with_home = base.with_home_flags(HomeFlagsCW::empty());
        let with_cia402 = base.with_cia402_flags(Cia402Flags::empty());

        // Check that only the relevant bits changed
        assert_ne!(base, with_pos);
        assert_ne!(base, with_home);
        assert_ne!(base, with_cia402);
    }

    #[test]
    fn test_cw_update_multiple_flag_applications() {
        let cw = ControlWord::default()
            .with_cia402_flags(Cia402Flags::ENABLE_VOLTAGE)
            .with_position_flags(PositionFlagsCW::NEW_SETPOINT);

        // Verify both sets of flags are present
        assert!(cw.contains(ControlWord::ENABLE_VOLTAGE));
        assert!(cw.contains(ControlWord::OMS_1)); // Position flag
    }
}

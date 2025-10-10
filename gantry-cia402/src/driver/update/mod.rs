pub mod publisher;

use thiserror::Error;

use crate::driver::{
    oms::{HomeFlags, PositionModeFlags},
    state::{Cia402Flags, Cia402State},
};

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Invalid state transition from {0:?} to {1:?}")]
    InvalidTransition(Cia402State, Cia402State),
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ControlWord: u16 {
        /// Bit 0: Switch on
        const SWITCH_ON              = 1 << 0;
        /// Bit 1: Enable voltage
        const ENABLE_VOLTAGE         = 1 << 1;
        /// Bit 2: Quick stop
        const QUICK_STOP             = 1 << 2;
        /// Bit 3: Enable operation
        const ENABLE_OPERATION       = 1 << 3;

        /// Bit 4: Operation mode specific
        const OP_MODE_SPECIFIC_1     = 1 << 4;
        /// Bit 5: Operation mode specific
        const OP_MODE_SPECIFIC_2     = 1 << 5;
        /// Bit 6: Operation mode specific
        const OP_MODE_SPECIFIC_3     = 1 << 6;

        /// Bit 7: Fault reset
        const FAULT_RESET            = 1 << 7;

        /// Bit 8: Halt
        const HALT                   = 1 << 8;
        /// Bit 9: Reserved
        const RESERVED_1             = 1 << 9;
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
    pub fn with_position_flags(self, flags: PositionModeFlags) -> Self {
        let mask = PositionModeFlags::all().bits();
        let new_bits = (self.bits() & !mask) | (flags.bits() & mask);
        ControlWord::from_bits_truncate(new_bits)
    }

    pub fn with_home_flags(self, flags: HomeFlags) -> Self {
        let mask = HomeFlags::all().bits();
        let new_bits = (self.bits() & !mask) | (flags.bits() & mask);
        ControlWord::from_bits_truncate(new_bits)
    }

    pub fn with_cia402_flags(self, flags: Cia402Flags) -> Self {
        let mask = Cia402Flags::all().bits();
        let new_bits = (self.bits() & !mask) | (flags.bits() & mask);
        ControlWord::from_bits_truncate(new_bits)
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
    fn test_with_position_flags() {
        let simple_cw = ControlWord::from_bits_truncate(0b0101010101010101);
        let flags = PositionModeFlags::empty();
        let simple_combined = simple_cw.with_position_flags(flags);
        let result = assert_eq!(simple_combined.bits(), 0b0101010000000101);

        let cw = ControlWord::from_bits_truncate(0b101010101001);
        let flags = PositionModeFlags::default();
        let combined = cw.with_position_flags(flags);
        let result = assert_eq!(combined.bits(), 0b0101010011010);
    }

    #[test]
    fn test_with_cia402_flags() {
        let cw = ControlWord::from_bits_truncate(0b1111111111111111);
        let flags = Cia402Flags::empty();
        let combined = cw.with_cia402_flags(flags);
        let result = assert_eq!(combined.bits(), 0b1111111110111000);
    }
}

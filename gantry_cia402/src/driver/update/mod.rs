use thiserror::Error;

use crate::{driver::state::Cia402State, od::oms::Setpoint};

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
const DEFAULT_CONTROL: ControlWord = ControlWord::empty();

pub const DEFAULT_UPDATE: Update = Update {
    controlword: None,
    setpoint: None,
    state: None,
};

#[derive(Debug, Clone)]
pub struct Update {
    pub controlword: Option<ControlWord>,
    pub setpoint: Option<Setpoint>,
    pub state: Option<Cia402State>,
}

impl Update {
    pub fn new(controlword: ControlWord, setpoint: Setpoint, state: Cia402State) -> Self {
        Self {
            controlword,
            setpoint,
            state,
        }
    }

    pub fn from_controlword(controlword: ControlWord) -> Self {
        Self {
            controlword,
            setpoint: None,
            state: None,
        }
    }

    pub fn from_setpoint(setpoint: Setpoint) -> Self {
        Self {
            controlword: None,
            setpoint,
            state: None,
        }
    }

    pub fn from_state(state: Cia402State) -> Self {
        Self {
            controlword: None,
            setpoint: None,
            state,
        }
    }
}

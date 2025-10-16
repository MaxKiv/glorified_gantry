pub mod error;
pub mod parse;
pub mod subscriber;

use std::time::Duration;

const COMMS_TIMEOUT: Duration = Duration::from_secs(1);

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct StatusWord: u16 {
        /// Bit 0: Ready to switch on
        const READY_TO_SWITCH_ON   = 1 << 0;
        /// Bit 1: Switched on
        const SWITCHED_ON          = 1 << 1;
        /// Bit 2: Operation enabled
        const OPERATION_ENABLED    = 1 << 2;
        /// Bit 3: Fault
        const FAULT                = 1 << 3;
        /// Bit 4: Voltage enabled
        const VOLTAGE_ENABLED      = 1 << 4;
        /// Bit 5: Quick stop disabled
        const QUICK_STOP_DISABLED  = 1 << 5;
        /// Bit 6: Switch on disabled
        const SWITCH_ON_DISABLED   = 1 << 6;
        /// Bit 7: Warning
        const WARNING              = 1 << 7;
        /// Bit 8: Manufacturer specific
        const MANUFACTURER_1       = 1 << 8;
        /// Bit 9: Remote (drive is under control via fieldbus)
        const REMOTE               = 1 << 9;
        /// Bit 10: Target reached (depends on operation mode)
        const TARGET_REACHED       = 1 << 10;
        /// Bit 11: Internal limit active
        const INTERNAL_LIMIT       = 1 << 11;
        /// Bit 12: Operation mode specific
        const OMS_1                = 1 << 12;
        /// Bit 13: Operation mode specific
        const OMS_2                = 1 << 13;
        /// Bit 14: Manufacturer specific
        const RESERVED             = 1 << 14;
        /// Bit 15: Controller is in the operation enabled state & the closed-loop mode is activated
        const CLOSED_LOOP_ACTIVE   = 1 << 15;
    }
}

impl Default for StatusWord {
    fn default() -> Self {
        StatusWord::empty()
    }
}

pub struct ActualPosition {
    pub value: i32,
}

pub struct ActualVelocity {
    pub value: i32,
}

pub struct ActualTorque {
    pub value: i16,
}

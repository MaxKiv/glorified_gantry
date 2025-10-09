pub mod error;
pub mod frame;
pub mod subscriber;

use std::time::Duration;

const COB_ID_SYNC: u16 = 0x80;
const COB_ID_TPDO1: u16 = 0x180;
const COB_ID_RPDO1: u16 = 0x200;
const COB_ID_TPDO2: u16 = 0x280;
const COB_ID_RPDO2: u16 = 0x300;
const COB_ID_TPDO3: u16 = 0x380;
const COB_ID_RPDO3: u16 = 0x400;
const COB_ID_TPDO4: u16 = 0x480;
const COB_ID_RPDO4: u16 = 0x500;
const COB_ID_SDO_RX: u16 = 0x600;
const COB_ID_SDO_TX: u16 = 0x580;
const COB_ID_HEARTBEAT: u16 = 0x700;

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
        /// Bit 5: Quick stop
        const QUICK_STOP           = 1 << 5;
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

        /// Bit 12: Manufacturer specific
        const MANUFACTURER_2       = 1 << 12;
        /// Bit 13: Manufacturer specific
        const MANUFACTURER_3       = 1 << 13;
        /// Bit 14: Manufacturer specific
        const MANUFACTURER_4       = 1 << 14;
        /// Bit 15: Manufacturer specific
        const MANUFACTURER_5       = 1 << 15;
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

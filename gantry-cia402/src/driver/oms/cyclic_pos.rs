use crate::driver::{event::MotorEvent, receiver::StatusWord};

#[derive(Clone, Debug)]
pub struct CyclicPositionSetpoint {
    pub abs_target: i32,
}

bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    /// Statusword OMS flags for Cyclic Synchronous Position mode
    /// See datasheet page 80
    pub struct CyclicPosFlagsSW: u16 {
        const DEVICE_IN_SYNC         = 1 << 8; // Is drive in sync with fieldbus?
        const RESERVED               = 1 << 10;
        const IS_FOLLOWING_TARGET    = 1 << 12; // Is drive using 0x607A (target position) as
                                                // setpoint?
        const HAS_FOLLOWING_ERROR    = 1 << 13;
    }
}

impl CyclicPosFlagsSW {
    pub fn from_status(sw: StatusWord) -> Self {
        Self::from_bits_truncate(sw.bits())
    }

    pub fn into_event(self) -> MotorEvent {
        // Datasheet page 80
        MotorEvent::CyclicPositionModeFeedback {
            device_in_sync: self.intersects(Self::DEVICE_IN_SYNC),
            is_following_target: self.intersects(Self::IS_FOLLOWING_TARGET),
            has_following_error: self.intersects(Self::HAS_FOLLOWING_ERROR),
        }
    }
}

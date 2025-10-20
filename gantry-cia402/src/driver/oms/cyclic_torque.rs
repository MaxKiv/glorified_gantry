use crate::driver::{event::MotorEvent, receiver::StatusWord};

#[derive(Clone, Debug)]
pub struct CyclicTorqueSetpoint {
    pub target: i16,
}

bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    /// Statusword OMS flags for Homing mode
    /// See datasheet page 80
    pub struct CyclicTorqueFlagsSW: u16 {
        const DEVICE_IN_SYNC         = 1 << 8;
        const RESERVED_1             = 1 << 10;
        const IS_FOLLOWING_TARGET    = 1 << 12;
        const RESERVED_2             = 1 << 13;
    }
}

impl CyclicTorqueFlagsSW {
    pub fn from_status(sw: StatusWord) -> Self {
        Self::from_bits_truncate(sw.bits())
    }

    pub fn into_event(self) -> MotorEvent {
        // Datasheet page 80
        MotorEvent::CyclicTorqueModeFeedback {
            device_in_sync: self.intersects(Self::DEVICE_IN_SYNC),
            is_following_target: self.intersects(Self::IS_FOLLOWING_TARGET),
        }
    }
}

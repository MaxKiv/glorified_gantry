use crate::driver::{event::MotorEvent, receiver::StatusWord};

#[derive(Clone, Debug)]
pub struct VelocitySetpoint {
    // TODO uom?
    pub target_velocity: i32,
}

bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    /// Statusword OMS flags for Homing mode
    /// See datasheet page 71
    pub struct VelocityFlagsSW: u16 {
        const SPEED_IS_ZERO          = 1 << 12;
        const DEVIATION_ERROR        = 1 << 13;
    }
}

impl VelocityFlagsSW {
    pub fn from_status(sw: StatusWord) -> Self {
        Self::from_bits_truncate(sw.bits())
    }

    pub fn into_event(self) -> MotorEvent {
        // Datasheet page 71
        MotorEvent::VelocityModeFeedback {
            speed_is_zero: self.intersects(Self::SPEED_IS_ZERO),
            deviation_error: self.intersects(Self::DEVIATION_ERROR),
        }
    }
}

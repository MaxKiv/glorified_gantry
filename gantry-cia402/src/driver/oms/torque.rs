use crate::driver::{event::MotorEvent, receiver::StatusWord};

#[derive(Clone, Debug)]
pub struct TorqueSetpoint {
    // TODO uom?
    pub target_torque: i16,
}

bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    /// Statusword OMS flags for Homing mode
    /// See datasheet page 71
    pub struct TorqueFlagsSW: u16 {
        const LIMIT_EXCEEDED          = 1 << 11;
    }
}

impl TorqueFlagsSW {
    pub fn from_status(sw: StatusWord) -> Self {
        Self::from_bits_truncate(sw.bits())
    }

    pub fn into_event(self) -> MotorEvent {
        // Datasheet page 71
        MotorEvent::TorqueModeFeedback {
            limit_exceeded: self.intersects(Self::LIMIT_EXCEEDED),
        }
    }
}

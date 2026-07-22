use crate::driver::{event::MotorEvent, receiver::StatusWord};

#[derive(Clone, Debug)]
pub struct TorqueSetpoint {
    pub target_torque: i16,
}

bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    /// Statusword OMS flags for Homing mode
    /// See datasheet page 71
    pub struct TorqueFlagsSW: u16 {
        const AXIS_BRAKED             = 1 << 8;
        const SETPOINT_REACHED        = 1 << 10;
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
            axis_braked: self.intersects(Self::AXIS_BRAKED),
            setpoint_reached: self.intersects(Self::SETPOINT_REACHED),
            limit_exceeded: self.intersects(Self::LIMIT_EXCEEDED),
        }
    }
}

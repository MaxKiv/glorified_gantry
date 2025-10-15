use crate::driver::{event::MotorEvent, receiver::StatusWord};

/// A Homing mode setpoint
#[derive(Clone, Debug)]
pub struct HomingSetpoint {
    pub flags: HomeFlagsCW,
}

bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    /// Controlword OMS flags for Homing mode
    pub struct HomeFlagsCW: u16 {
        const NEW_SETPOINT       = 1 << 4; // Bit 4: Rising edge triggers start of movement
    }
}

impl Default for HomeFlagsCW {
    fn default() -> Self {
        HomeFlagsCW::NEW_SETPOINT
    }
}

bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    /// Statusword OMS flags for Homing mode
    /// See datasheet page 71
    pub struct HomeFlagsSW: u16 {
        const AT_HOME             = 1 << 10;
        const HOMING_COMPLETED    = 1 << 12;
        const HOMING_ERROR        = 1 << 13;
    }
}

impl HomeFlagsSW {
    pub fn from_status(sw: StatusWord) -> Self {
        Self::from_bits_truncate(sw.bits())
    }

    pub fn into_event(self) -> MotorEvent {
        // Datasheet page 71
        MotorEvent::HomingFeedback {
            at_home: self.intersects(Self::AT_HOME),
            homing_completed: self.intersects(Self::HOMING_COMPLETED),
            homing_error: self.intersects(Self::HOMING_ERROR),
        }
    }
}

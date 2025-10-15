use crate::driver::{event::MotorEvent, receiver::StatusWord};

pub const STARTUP_POSITIONMODE_SETPOINT: PositionSetpoint = PositionSetpoint {
    flags: PositionModeFlagsCW::empty(),
    target: 0,
    profile_velocity: 0,
};

// Controlword OMS flags for Profile Position mode
bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    pub struct PositionModeFlagsCW: u16 {
        const NEW_SETPOINT       = 1 << 4; // Bit 4: Rising edge triggers start of movement
        const CHANGE_IMMEDIATELY = 1 << 5; // Bit 5: Should the motor instantly adapt to the new setpoint, or first reach the previous target?
        const RELATIVE           = 1 << 6; // Bit 6: Interpret this target as a relative position, see 0x60F2
        const HALT               = 1 << 8; // Bit 8: Halt the motor
        const CHANGE_ON_SETPOINT = 1 << 9; // Bit 9: Should the motor have velocity 0 at target position? see page 60
    }
}

impl Default for PositionModeFlagsCW {
    fn default() -> Self {
        PositionModeFlagsCW::NEW_SETPOINT// By default start movement when new setpoint is given
        | PositionModeFlagsCW::CHANGE_IMMEDIATELY  // By default instantly adopt new setpoint, overriding old
        & !(PositionModeFlagsCW::RELATIVE)         // By default interpret target position as absolute position
        & !(PositionModeFlagsCW::HALT)             // By default do not halt
        | PositionModeFlagsCW::CHANGE_ON_SETPOINT // By default have zero velocity when reaching setpoint
    }
}
impl PositionModeFlagsCW {
    pub fn absolute() -> Self {
        Self::default()
    }

    pub fn relative() -> Self {
        Self::default() | PositionModeFlagsCW::RELATIVE
    }

    pub fn halt() -> Self {
        PositionModeFlagsCW::NEW_SETPOINT// By default start movement when new setpoint is given
        | PositionModeFlagsCW::CHANGE_IMMEDIATELY  // By default instantly adopt new setpoint, overriding old
        | PositionModeFlagsCW::CHANGE_ON_SETPOINT // By default have zero velocity when reaching setpoint
        | PositionModeFlagsCW::HALT // Stop!
    }
}

#[derive(Clone, Debug)]
pub struct PositionSetpoint {
    pub flags: PositionModeFlagsCW,
    pub target: i32,
    pub profile_velocity: u32,
}

bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    /// Statusword OMS flags for Homing mode
    /// See datasheet page 71
    pub struct PositionFlagsSW: u16 {
        const TARGET_REACHED          = 1 << 10;
        const LIMIT_EXCEEDED          = 1 << 11;
        const SETPOINT_ACKNOWLEGDE    = 1 << 12;
        const FOLLOWING_ERROR         = 1 << 13;
    }
}

impl PositionFlagsSW {
    pub fn from_status(sw: StatusWord) -> Self {
        Self::from_bits_truncate(sw.bits())
    }

    pub fn into_event(self) -> MotorEvent {
        // Datasheet page 71
        MotorEvent::PositionModeFeedback {
            target_reached: self.intersects(Self::TARGET_REACHED),
            limit_exceeded: self.intersects(Self::LIMIT_EXCEEDED),
            setpoint_acknowlegde: self.intersects(Self::SETPOINT_ACKNOWLEGDE),
            following_error: self.intersects(Self::FOLLOWING_ERROR),
        }
    }
}

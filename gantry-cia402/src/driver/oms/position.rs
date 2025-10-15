pub const STARTUP_POSITIONMODE_SETPOINT: PositionSetpoint = PositionSetpoint {
    flags: PositionModeFlags::empty(),
    target: 0,
    profile_velocity: 0,
};

bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    pub struct PositionModeFlags: u16 {
        const NEW_SETPOINT       = 1 << 4; // Bit 4: Rising edge triggers start of movement
        const CHANGE_IMMEDIATELY = 1 << 5; // Bit 5: Should the motor instantly adapt to the new setpoint, or first reach the previous target?
        const RELATIVE           = 1 << 6; // Bit 6: Interpret this target as a relative position, see 0x60F2
        const HALT               = 1 << 8; // Bit 8: Halt the motor
        const CHANGE_ON_SETPOINT = 1 << 9; // Bit 9: Should the motor have velocity 0 at target position? see page 60
    }
}

impl Default for PositionModeFlags {
    fn default() -> Self {
        PositionModeFlags::NEW_SETPOINT// By default start movement when new setpoint is given
        | PositionModeFlags::CHANGE_IMMEDIATELY  // By default instantly adopt new setpoint, overriding old
        & !(PositionModeFlags::RELATIVE)         // By default interpret target position as absolute position
        & !(PositionModeFlags::HALT)             // By default do not halt
        | PositionModeFlags::CHANGE_ON_SETPOINT // By default have zero velocity when reaching setpoint
    }
}
impl PositionModeFlags {
    pub fn absolute() -> Self {
        Self::default()
    }

    pub fn relative() -> Self {
        Self::default() | PositionModeFlags::RELATIVE
    }

    pub fn halt() -> Self {
        PositionModeFlags::NEW_SETPOINT// By default start movement when new setpoint is given
        | PositionModeFlags::CHANGE_IMMEDIATELY  // By default instantly adopt new setpoint, overriding old
        | PositionModeFlags::CHANGE_ON_SETPOINT // By default have zero velocity when reaching setpoint
        | PositionModeFlags::HALT // Stop!
    }
}

#[derive(Clone, Debug)]
pub struct PositionSetpoint {
    pub flags: PositionModeFlags,
    pub target: i32,
    pub profile_velocity: u32,
}

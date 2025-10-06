pub mod bitmask;

pub struct ODEntry {
    pub index: u16,
    pub sub_index: u8,
    pub default: Option<u32>,
}

impl ODEntry {
    pub const fn new(index: u16, sub_index: u8) -> Self {
        Self {
            index,
            sub_index,
            default: None,
        }
    }

    pub const fn with_default(index: u16, sub_index: u8, default: u32) -> Self {
        Self {
            index,
            sub_index,
            default: Some(default),
        }
    }
}

pub struct ObjectDictionary {}

impl ObjectDictionary {
    // General
    pub const DEVICE_TYPE: ODEntry = ODEntry::new(0x1000, 0x0);
    pub const CONTROL_WORD: ODEntry = ODEntry::new(0x6040, 0x0);
    pub const STATUS_WORD: ODEntry = ODEntry::new(0x6041, 0x0);

    /// Heartbeat produce time in milliseconds
    pub const PRODUCER_HEARTBEAT_TIME: ODEntry = ODEntry::new(0x1017, 0x0);

    // Feedback related
    pub const POSITION_ACTUAL_VALUE: ODEntry = ODEntry::new(0x6064, 0x00);
    pub const VELOCITY_ACTUAL_VALUE: ODEntry = ODEntry::new(0x606C, 0x00);
    pub const TORQUE_ACTUAL_VALUE: ODEntry = ODEntry::new(0x6077, 0x00);

    /// Mode of operation related
    /// 1 = Profile Position, 3 = Profile Velocity, 4 = Profile Torque, 6 = Homing
    pub const SET_OPERATION_MODE: ODEntry = ODEntry::new(0x6060, 0x0);
    pub const GET_OPERATION_MODE: ODEntry = ODEntry::new(0x6061, 0x0);

    /// Targets
    pub const SET_TARGET_POSITION: ODEntry = ODEntry::new(0x607A, 0x00);
    pub const SET_TARGET_VELOCITY: ODEntry = ODEntry::new(0x60FF, 0x00);
    pub const SET_TARGET_TORQUE: ODEntry = ODEntry::new(0x6071, 0x00);

    // Position mode related
    pub const SOFTWARE_POSITION_LIMIT: ODEntry = ODEntry::new(0x607D, 0x00);

    pub const HOME_OFFSET: ODEntry = ODEntry::new(0x607C, 0x00);

    pub const POSITION_RANGE_LIMIT: ODEntry = ODEntry::new(0x607B, 0x00);

    pub const POLARITY: ODEntry = ODEntry::new(0x607E, 0x00);

    pub const PROFILE_VELOCITY: ODEntry = ODEntry::new(0x6081, 0x00);

    pub const END_VELOCITY: ODEntry = ODEntry::new(0x6082, 0x00);

    pub const PROFILE_ACCELERATION: ODEntry = ODEntry::new(0x6083, 0x00);

    pub const PROFILE_DECELERATION: ODEntry = ODEntry::new(0x6084, 0x00);

    pub const QUICK_STOP_DECELERATION: ODEntry = ODEntry::new(0x6085, 0x00);

    pub const MOTION_PROFILE_TYPE: ODEntry = ODEntry::new(0x6086, 0x00);

    pub const MAX_ACCELERATION: ODEntry = ODEntry::new(0x60C5, 0x00);

    pub const MAX_DECELERATION: ODEntry = ODEntry::new(0x60C6, 0x00);

    pub const PROFILE_JERK: ODEntry = ODEntry::new(0x60A4, 0x00);

    pub const POSITIONING_OPTION_CODE: ODEntry = ODEntry::new(0x60F2, 0x00);

    // PDO related (datasheet page 118)
    // NOTE: these only work when in NMT::PreOperational

    /// Base index for the RPDO configuration
    /// e.g. to configure RPDO #3 communication take base + (3-1) = 0x1402
    pub const RPDO_COMMUNICATION_PARAMETER_BASE_INDEX: u16 = 0x1400;
    pub const RPDO_MAPPING_PARAMETER_BASE_INDEX: u16 = 0x1600;
    pub const TPDO_COMMUNICATION_PARAMETER_BASE_INDEX: u16 = 0x1800;
    pub const TPDO_MAPPING_PARAMETER_BASE_INDEX: u16 = 0x1A00;

    // Unit related

    /// Combines the position mode unit and exponent
    /// Default value is 'tenths of degrees' (3600 = 1 full rotation)
    pub const SI_UNIT_POSITION: ODEntry = ODEntry::with_default(0x60A8, 0x0, 0xFF410000);

    /// Combines the velocity mode units for position and time, and the exponent
    /// Default value is 'revolutions per minute'
    pub const SI_UNIT_SPEED: ODEntry = ODEntry::with_default(0x60A9, 0x0, 0x00B44700);

    pub const POSITION_MODE_MINIMUM_PARAMS: &[ODEntry] = &[
        Self::SET_TARGET_POSITION,
        Self::SOFTWARE_POSITION_LIMIT,
        Self::HOME_OFFSET,
        Self::POSITION_RANGE_LIMIT,
        Self::POLARITY,
        Self::PROFILE_VELOCITY,
        Self::END_VELOCITY,
        Self::PROFILE_ACCELERATION,
        Self::PROFILE_DECELERATION,
        Self::QUICK_STOP_DECELERATION,
        Self::MOTION_PROFILE_TYPE,
        Self::MAX_ACCELERATION,
        Self::MAX_DECELERATION,
        Self::PROFILE_JERK,
        Self::POSITIONING_OPTION_CODE,
    ];
}

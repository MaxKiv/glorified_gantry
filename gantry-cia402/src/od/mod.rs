use access::AccessType;

use crate::od::{entry::ODEntry, mappable::MappableType, value::ODValue};

pub mod access;
pub mod entry;
pub mod mappable;
pub mod value;

// index: u16, sub_index: u8, access: AccessType, pdo_mappable: bool, default: ODValue

/// Device Type — identifies the device profile
pub const DEVICE_TYPE: ODEntry = ODEntry::new(
    0x1000,
    0x00,
    AccessType::ReadOnly,
    MappableType::None,
    ODValue::U32(0x0004_0192), // CiA 402 drive
);

/// Controlword — control state machine & motion commands
pub const CONTROL_WORD: ODEntry = ODEntry::new(
    0x6040,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U16(0x0000),
);

/// Statusword — drive state and feedback
pub const STATUS_WORD: ODEntry = ODEntry::new(
    0x6041,
    0x00,
    AccessType::ReadOnly,
    MappableType::TPDO,
    ODValue::U16(0x0000),
);

/// Heartbeat producer time in [ms]
/// Page 121
pub const PRODUCER_HEARTBEAT_TIME: ODEntry = ODEntry::new(
    0x1017,
    0x00,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U16(0), // By default send no heartbeat
);

/// Actual position value [counts]
pub const POSITION_ACTUAL_VALUE: ODEntry = ODEntry::new(
    0x6064,
    0x00,
    AccessType::ReadOnly,
    MappableType::TPDO,
    ODValue::I32(0),
);

/// Actual velocity value [counts/s]
pub const VELOCITY_ACTUAL_VALUE: ODEntry = ODEntry::new(
    0x606C,
    0x00,
    AccessType::ReadOnly,
    MappableType::TPDO,
    ODValue::I32(0),
);

/// Actual torque value [0.1 % of nominal torque]
pub const TORQUE_ACTUAL_VALUE: ODEntry = ODEntry::new(
    0x6077,
    0x00,
    AccessType::ReadOnly,
    MappableType::TPDO,
    ODValue::I16(0),
);

/// Mode of operation (set)
/// 1 = Profile Position, 3 = Profile Velocity, 4 = Profile Torque, 6 = Homing
pub const SET_OPERATION_MODE: ODEntry = ODEntry::new(
    0x6060,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I8(1),
);

/// Mode of operation (get)
pub const GET_OPERATION_MODE: ODEntry = ODEntry::new(
    0x6061,
    0x00,
    AccessType::ReadOnly,
    MappableType::RPDO,
    ODValue::I8(1),
);

/// Target position [counts] (default 3600 counts = 1 rev)
pub const SET_TARGET_POSITION: ODEntry = ODEntry::new(
    0x607A,
    0x00,
    AccessType::ReadWrite,
    MappableType::TPDO,
    ODValue::I32(0x0000_0FA0),
);

/// Target velocity [counts/s]
pub const SET_TARGET_VELOCITY: ODEntry = ODEntry::new(
    0x60FF,
    0x00,
    AccessType::ReadWrite,
    MappableType::TPDO,
    ODValue::I32(0),
);

/// Target torque [0.1 % of nominal torque]
pub const SET_TARGET_TORQUE: ODEntry = ODEntry::new(
    0x6071,
    0x00,
    AccessType::ReadWrite,
    MappableType::TPDO,
    ODValue::I16(0),
);

/// Software position limit - defines the limit positions relative to the reference point of the
/// application in user defined units
pub const SOFTWARE_POSITION_LIMIT: ODEntry = ODEntry::new(
    0x607D,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::Array(3),
);

/// Software Position range limit — subindex 1 = min, subindex 2 = max
pub const SOFTWARE_POSITION_RANGE_LIMIT_MIN: ODEntry = ODEntry::new(
    0x607D,
    0x01,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0),
);

/// Software Position range limit — subindex 1 = min, subindex 2 = max
pub const SOFTWARE_POSITION_RANGE_LIMIT_MAX: ODEntry = ODEntry::new(
    0x607D,
    0x02,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0),
);

/// Contains the minimum and maximum position limit in user defined units
pub const POSITION_LIIMT: ODEntry = ODEntry::new(
    0x607B,
    0x00,
    AccessType::ReadOnly,
    MappableType::RPDO,
    ODValue::Array(3),
);

/// Position range limit — subindex 1 = min, subindex 2 = max
pub const POSITION_RANGE_LIMIT_MIN: ODEntry = ODEntry::new(
    0x607B,
    0x01,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0),
);

/// Position range limit — subindex 1 = min, subindex 2 = max
pub const POSITION_RANGE_LIMIT_MAX: ODEntry = ODEntry::new(
    0x607B,
    0x02,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0),
);

/// Home offset — Specifies the difference between the zero position of the controller
/// and the reference point of the machine in user-defined units [counts]
/// Applied after homing completes
pub const HOME_OFFSET: ODEntry = ODEntry::new(
    0x607C,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0),
);

/// Polarity — inverts direction of motion or sensor inputs
pub const POLARITY: ODEntry = ODEntry::new(
    0x607E,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U8(0),
);

/// Profile velocity — desired constant velocity in Profile Position/Velocity modes [counts/s]
pub const PROFILE_VELOCITY: ODEntry = ODEntry::new(
    0x6081,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x01F4),
);

/// End velocity — used for homing or interpolated motion [counts/s]
pub const END_VELOCITY: ODEntry = ODEntry::new(
    0x6082,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0),
);

/// Profile acceleration — acceleration during motion [counts/s²]
pub const PROFILE_ACCELERATION: ODEntry = ODEntry::new(
    0x6083,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x01F4),
);

/// Profile deceleration — deceleration during motion [counts/s²]
pub const PROFILE_DECELERATION: ODEntry = ODEntry::new(
    0x6084,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x01F4),
);

/// Quick stop deceleration — deceleration used during quick stop [counts/s²]
pub const QUICK_STOP_DECELERATION: ODEntry = ODEntry::new(
    0x6085,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x1388),
);

/// Motion profile type — defines velocity profile shape
/// 0 = trapezoidal, 1 = sinusoidal
pub const MOTION_PROFILE_TYPE: ODEntry = ODEntry::new(
    0x6086,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I16(0),
);

/// Max acceleration [counts/s²]
pub const MAX_ACCELERATION: ODEntry = ODEntry::new(
    0x60C5,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x1388),
);

/// Max deceleration [counts/s²]
pub const MAX_DECELERATION: ODEntry = ODEntry::new(
    0x60C6,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x1388),
);

/// Profile jerk — rate of change of acceleration [counts/s³]
pub const PROFILE_JERK: ODEntry = ODEntry::new(
    0x60A4,
    0x00,
    AccessType::ReadOnly,
    MappableType::None,
    ODValue::Array(5),
);

pub const PROFILE_JERK_BEGIN_ACCEL: ODEntry = ODEntry::new(
    0x60A4,
    0x01,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U32(0x03E8),
);

pub const PROFILE_JERK_BEGIN_DECEL: ODEntry = ODEntry::new(
    0x60A4,
    0x02,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U32(0x03E8),
);

pub const PROFILE_JERK_END_ACCEL: ODEntry = ODEntry::new(
    0x60A4,
    0x03,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U32(0x03E8),
);

pub const PROFILE_JERK_END_DECEL: ODEntry = ODEntry::new(
    0x60A4,
    0x04,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U32(0x03E8),
);

/// Positioning option code — defines motion termination and rounding behavior
/// Only used when doing Relative Profile Position movements
/// Page 394
pub const POSITIONING_OPTION_CODE: ODEntry = ODEntry::new(
    0x60F2,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U16(1), // Position movements are executed relative to the preset value (or output) of the ramp generator
);

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
/// Page 378
pub const SI_UNIT_POSITION: ODEntry = ODEntry::new(
    0x60A8,
    0x00,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U32(0xFF410000), // Combined value [tenth of degrees], look at page 378
);

/// Combines the velocity mode units for position and time, and the exponent
/// Default value is 'revolutions per minute'
/// Page 379
pub const SI_UNIT_SPEED: ODEntry = ODEntry::new(
    0x60A9,
    0x00,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U32(0x00B447000), // Combined value [revolutions per minute], look at page 379
);

pub const POSITION_MODE_MINIMUM_PARAMS: &[ODEntry] = &[
    SET_TARGET_POSITION,
    SOFTWARE_POSITION_LIMIT,
    HOME_OFFSET,
    POSITION_RANGE_LIMIT_MIN,
    POSITION_RANGE_LIMIT_MAX,
    POLARITY,
    PROFILE_VELOCITY,
    END_VELOCITY,
    PROFILE_ACCELERATION,
    PROFILE_DECELERATION,
    QUICK_STOP_DECELERATION,
    MOTION_PROFILE_TYPE,
    MAX_ACCELERATION,
    MAX_DECELERATION,
    PROFILE_JERK,
    POSITIONING_OPTION_CODE,
];

use access::AccessType;
use once_cell::sync::Lazy;

use crate::{
    driver::startup::home::HomingMethods,
    od::{entry::ODEntry, mappable::MappableType, value::ODValue},
};
use heapless::index_map::FnvIndexMap;

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

/// Homing method — Defines which homing procedure the device should use.
/// See CiA 402 Table 46 for method codes (e.g. 1 = Home on negative limit, 33 = Home on positive limit, etc.)
/// [unitless]
pub const HOMING_METHOD: ODEntry = ODEntry::new(
    0x6098,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I8(HomingMethods::IndexOnly.as_i8()), // Home on current position
);

/// Speed during search for switch — Speed used while seeking the limit or home switch
/// during the first phase of the homing sequence [counts/s]
pub const HOMING_SPEED_SWITCH_SEARCH: ODEntry = ODEntry::new(
    0x6099,
    0x01,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x32),
);

/// Speed during search for zero — Speed used for the fine search phase
/// after switch detection, to locate the mechanical or encoder zero [counts/s]
pub const HOMING_SPEED_ZERO_SEARCH: ODEntry = ODEntry::new(
    0x6099,
    0x02,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x0A),
);

/// Maximum motor speed — Defines the motor’s absolute maximum velocity
/// the controller may command [counts/s]
pub const MAX_MOTOR_SPEED: ODEntry = ODEntry::new(
    0x6080,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x7530),
);

/// Homing acceleration — Acceleration (and deceleration) to use during the homing procedure [counts/s²]
pub const HOMING_ACCELERATION: ODEntry = ODEntry::new(
    0x609A,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x1F4),
);

/// Minimum current for block detection — Threshold current above which the motor
/// is considered blocked [mA]
pub const BLOCK_DETECTION_MIN_CURRENT: ODEntry = ODEntry::new(
    0x203A,
    0x01,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0x41A), // 1050 mA
);

/// Period of blocking — Time duration the motor continues to run after
/// detecting a block condition [ms]
pub const BLOCK_DETECTION_PERIOD: ODEntry = ODEntry::new(
    0x203A,
    0x02,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0xC8), // 200ms
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

/// Minimum set of Object Dictionary entries required for Profile Position
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

/// Minimum set of Object Dictionary entries required for Homing Mode (CiA 402 § 6.5.1.5)
pub const HOMING_MODE_MINIMUM_PARAMS: &[ODEntry] = &[
    HOME_OFFSET,                 // 607Ch
    HOMING_METHOD,               // 6098h
    HOMING_SPEED_SWITCH_SEARCH,  // 6099h:01h
    HOMING_SPEED_ZERO_SEARCH,    // 6099h:02h
    MAX_MOTOR_SPEED,             // 6080h
    HOMING_ACCELERATION,         // 609Ah
    BLOCK_DETECTION_MIN_CURRENT, // 203Ah:01h
    BLOCK_DETECTION_PERIOD,      // 203Ah:02h
];

pub const FULL_OBJECT_DICTIONARY: &[ODEntry] = &[
    DEVICE_TYPE,
    CONTROL_WORD,
    STATUS_WORD,
    PRODUCER_HEARTBEAT_TIME,
    POSITION_ACTUAL_VALUE,
    VELOCITY_ACTUAL_VALUE,
    TORQUE_ACTUAL_VALUE,
    SET_OPERATION_MODE,
    GET_OPERATION_MODE,
    SET_TARGET_POSITION,
    SET_TARGET_VELOCITY,
    SET_TARGET_TORQUE,
    SOFTWARE_POSITION_LIMIT,
    SOFTWARE_POSITION_RANGE_LIMIT_MIN,
    SOFTWARE_POSITION_RANGE_LIMIT_MAX,
    POSITION_LIIMT,
    POSITION_RANGE_LIMIT_MIN,
    POSITION_RANGE_LIMIT_MAX,
    HOME_OFFSET,
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
    PROFILE_JERK_BEGIN_ACCEL,
    PROFILE_JERK_BEGIN_DECEL,
    PROFILE_JERK_END_ACCEL,
    PROFILE_JERK_END_DECEL,
    POSITIONING_OPTION_CODE,
    SI_UNIT_POSITION,
    SI_UNIT_SPEED,
];

#[derive(Eq, PartialEq, Hash)]
pub struct ODIdx {
    pub index: u16,
    pub sub_index: u8,
}

static OD_LOOKUP: Lazy<FnvIndexMap<ODIdx, &ODEntry, 64>> = Lazy::new(|| {
    let mut m = FnvIndexMap::new();
    for entry in FULL_OBJECT_DICTIONARY {
        m.insert(
            ODIdx {
                index: entry.index,
                sub_index: entry.sub_index,
            },
            entry,
        );
    }
    m
});

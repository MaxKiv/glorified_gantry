use access::AccessType;
use once_cell::sync::Lazy;

use crate::{
    canopen::od::{
        entry::{ODEntry, PdoSemantic},
        mappable::MappableType,
        value::ODValue,
    },
    oms::home::HomingMethods,
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
    PdoSemantic::DeviceType,
);

/// Device Type — identifies the device profile
pub const DEVICE_NAME: ODEntry = ODEntry::new(
    0x1008,
    0x00,
    AccessType::ReadOnly,
    MappableType::None,
    ODValue::VisibleString([0u8; 8]),
    PdoSemantic::DeviceName,
);

/// Controlword — control state machine & motion commands
pub const CONTROL_WORD: ODEntry = ODEntry::new(
    0x6040,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U16(0x0000),
    PdoSemantic::Controlword,
);

/// Statusword — drive state and feedback
pub const STATUS_WORD: ODEntry = ODEntry::new(
    0x6041,
    0x00,
    AccessType::ReadOnly,
    MappableType::TPDO,
    ODValue::U16(0x0000),
    PdoSemantic::Statusword,
);

/// Heartbeat producer time in [ms]
/// Page 121
pub const PRODUCER_HEARTBEAT_TIME: ODEntry = ODEntry::new(
    0x1017,
    0x00,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U16(0), // By default send no heartbeat
    PdoSemantic::Other,
);

/// Actual position value [counts]
pub const POSITION_ACTUAL_VALUE: ODEntry = ODEntry::new(
    0x6064,
    0x00,
    AccessType::ReadOnly,
    MappableType::TPDO,
    ODValue::I32(0),
    PdoSemantic::ActualPosition,
);

/// Actual velocity value [counts/s]
pub const VELOCITY_ACTUAL_VALUE: ODEntry = ODEntry::new(
    0x606C,
    0x00,
    AccessType::ReadOnly,
    MappableType::TPDO,
    ODValue::I32(0),
    PdoSemantic::ActualVelocity,
);

/// Actual torque value [0.1 % of nominal torque]
pub const TORQUE_ACTUAL_VALUE: ODEntry = ODEntry::new(
    0x6077,
    0x00,
    AccessType::ReadOnly,
    MappableType::TPDO,
    ODValue::I16(0),
    PdoSemantic::ActualTorque,
);

/// Contains the current following error in user-defined units
pub const FOLLOWING_ERROR_ACTUAL_VALUE: ODEntry = ODEntry::new(
    0x60F4,
    0x00,
    AccessType::ReadOnly,
    MappableType::TPDO,
    ODValue::I32(0),
    PdoSemantic::Other,
);

/// Mode of operation (set)
/// 1 = Profile Position, 3 = Profile Velocity, 4 = Profile Torque, 6 = Homing
pub const SET_OPERATION_MODE: ODEntry = ODEntry::new(
    0x6060,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I8(1),
    PdoSemantic::TargetOperationMode,
);

/// Mode of operation (get)
pub const GET_OPERATION_MODE: ODEntry = ODEntry::new(
    0x6061,
    0x00,
    AccessType::ReadOnly,
    MappableType::TPDO,
    ODValue::I8(1),
    PdoSemantic::ActualOperationMode,
);

/// Position Window
/// Determines when TARGET_REACHED is emitted by drive together with 0x6068
/// 0 = strictest, no deviation allowed before emitting TARGET_REACHED
/// 0xFFFFFFFF = disable TARGET_REACHED emission altogether
pub const POSITION_WINDOW: ODEntry = ODEntry::new(
    0x6067,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0),
    PdoSemantic::Other,
);

/// Target position [counts] (default 3600 counts = 1 rev)
pub const SET_TARGET_POSITION: ODEntry = ODEntry::new(
    0x607A,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0x0000_0FA0),
    PdoSemantic::TargetPosition,
);

/// Target velocity [counts/s]
pub const SET_TARGET_VELOCITY: ODEntry = ODEntry::new(
    0x60FF,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0),
    PdoSemantic::TargetVelocity,
);

/// Target torque — Desired torque setpoint in thousandths of the maximum torque.
/// 1000 corresponds to the maximum torque (rated current).
pub const SET_TARGET_TORQUE: ODEntry = ODEntry::new(
    0x6071,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I16(0),
    PdoSemantic::TargetTorque,
);

/// Software position limit - defines the limit positions relative to the reference point of the
/// application in user defined units
pub const SOFTWARE_POSITION_LIMIT: ODEntry = ODEntry::new(
    0x607D,
    0x00,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::Array(3),
    PdoSemantic::Other,
);

/// Software Position range limit — subindex 1 = min, subindex 2 = max
pub const SOFTWARE_POSITION_RANGE_LIMIT_MIN: ODEntry = ODEntry::new(
    0x607D,
    0x01,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0),
    PdoSemantic::Other,
);

/// Software Position range limit — subindex 1 = min, subindex 2 = max
pub const SOFTWARE_POSITION_RANGE_LIMIT_MAX: ODEntry = ODEntry::new(
    0x607D,
    0x02,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0),
    PdoSemantic::Other,
);

/// Contains the minimum and maximum position limit in user defined units
pub const POSITION_LIIMT: ODEntry = ODEntry::new(
    0x607B,
    0x00,
    AccessType::ReadOnly,
    MappableType::None,
    ODValue::Array(3),
    PdoSemantic::Other,
);

/// Position range limit — subindex 1 = min, subindex 2 = max
pub const POSITION_RANGE_LIMIT_MIN: ODEntry = ODEntry::new(
    0x607B,
    0x01,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0),
    PdoSemantic::Other,
);

/// Position range limit — subindex 1 = min, subindex 2 = max
pub const POSITION_RANGE_LIMIT_MAX: ODEntry = ODEntry::new(
    0x607B,
    0x02,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0),
    PdoSemantic::Other,
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
    PdoSemantic::Other,
);

/// Polarity — inverts direction of motion or sensor inputs
pub const POLARITY: ODEntry = ODEntry::new(
    0x607E,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U8(0),
    PdoSemantic::Other,
);

/// Profile velocity — desired constant velocity in Profile Position/Velocity modes [counts/s]
pub const PROFILE_VELOCITY: ODEntry = ODEntry::new(
    0x6081,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x01F4),
    PdoSemantic::ProfileVelocity,
);

/// End velocity — used for homing or interpolated motion [counts/s]
pub const END_VELOCITY: ODEntry = ODEntry::new(
    0x6082,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0),
    PdoSemantic::Other,
);

/// Profile acceleration — acceleration during motion [counts/s²]
pub const PROFILE_ACCELERATION: ODEntry = ODEntry::new(
    0x6083,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x01F4),
    PdoSemantic::ProfileAcceleration,
);

/// Profile deceleration — deceleration during motion [counts/s²]
pub const PROFILE_DECELERATION: ODEntry = ODEntry::new(
    0x6084,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x01F4),
    PdoSemantic::ProfileDecceleration,
);

/// Quick stop deceleration — deceleration used during quick stop [counts/s²]
pub const QUICK_STOP_DECELERATION: ODEntry = ODEntry::new(
    0x6085,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x1388),
    PdoSemantic::Other,
);

/// Motion profile type — defines velocity profile shape
/// 0 = trapezoidal, 1 = sinusoidal
pub const MOTION_PROFILE_TYPE: ODEntry = ODEntry::new(
    0x6086,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I16(0),
    PdoSemantic::Other,
);

/// Max acceleration [counts/s²] for profile modes
pub const MAX_ACCELERATION: ODEntry = ODEntry::new(
    0x60C5,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x1388),
    PdoSemantic::Other,
);

/// Max deceleration [counts/s²] for profile modes
pub const MAX_DECELERATION: ODEntry = ODEntry::new(
    0x60C6,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x1388),
    PdoSemantic::Other,
);

/// Profile jerk — rate of change of acceleration [counts/s³]
pub const PROFILE_JERK: ODEntry = ODEntry::new(
    0x60A4,
    0x00,
    AccessType::ReadOnly,
    MappableType::None,
    ODValue::Array(5),
    PdoSemantic::Other,
);

pub const PROFILE_JERK_BEGIN_ACCEL: ODEntry = ODEntry::new(
    0x60A4,
    0x01,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U32(0x03E8),
    PdoSemantic::Other,
);

pub const PROFILE_JERK_BEGIN_DECEL: ODEntry = ODEntry::new(
    0x60A4,
    0x02,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U32(0x03E8),
    PdoSemantic::Other,
);

pub const PROFILE_JERK_END_ACCEL: ODEntry = ODEntry::new(
    0x60A4,
    0x03,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U32(0x03E8),
    PdoSemantic::Other,
);

pub const PROFILE_JERK_END_DECEL: ODEntry = ODEntry::new(
    0x60A4,
    0x04,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::U32(0x03E8),
    PdoSemantic::Other,
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
    PdoSemantic::Other,
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
    PdoSemantic::Other,
);

/// Speed during search for switch — Speed used while seeking the limit or home switch
/// during the first phase of the homing sequence [counts/s]
pub const HOMING_SPEED_SWITCH_SEARCH: ODEntry = ODEntry::new(
    0x6099,
    0x01,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x32),
    PdoSemantic::Other,
);

/// Speed during search for zero — Speed used for the fine search phase
/// after switch detection, to locate the mechanical or encoder zero [counts/s]
pub const HOMING_SPEED_ZERO_SEARCH: ODEntry = ODEntry::new(
    0x6099,
    0x02,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x0A),
    PdoSemantic::Other,
);

/// Maximum motor speed — Defines the motor’s absolute maximum velocity
/// the controller may command [counts/s]
pub const MAX_MOTOR_SPEED: ODEntry = ODEntry::new(
    0x6080,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x7530),
    PdoSemantic::Other,
);

/// Homing acceleration — Acceleration (and deceleration) to use during the homing procedure [counts/s²]
pub const HOMING_ACCELERATION: ODEntry = ODEntry::new(
    0x609A,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0x1F4),
    PdoSemantic::Other,
);

/// Minimum current for block detection — Threshold current above which the motor
/// is considered blocked [mA]
pub const BLOCK_DETECTION_MIN_CURRENT: ODEntry = ODEntry::new(
    0x203A,
    0x01,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0x41A), // 1050 mA
    PdoSemantic::Other,
);

/// Period of blocking — Time duration the motor continues to run after
/// detecting a block condition [ms]
pub const BLOCK_DETECTION_PERIOD: ODEntry = ODEntry::new(
    0x203A,
    0x02,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::I32(0xC8), // 200ms
    PdoSemantic::Other,
);

/// Max torque — Limit for torque during the entire ramp (accelerate, maintain, decelerate)
/// expressed in thousandths of maximum torque.
pub const MAX_TORQUE: ODEntry = ODEntry::new(
    0x6072,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U16(0x64), // 100 = 1/10 of max rated torque
    PdoSemantic::Other,
);

/// Max current — Maximum current in thousandths of rated current.
/// The minimum of this and 6072h limits the torque in 6071h.
pub const MAX_CURRENT: ODEntry = ODEntry::new(
    0x6073,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U16(1000), // 1000 ‰ = rated current
    PdoSemantic::Other,
);

/// Torque demand — Current output torque (from ramp generator) in thousandths of max torque.
pub const TORQUE_DEMAND: ODEntry = ODEntry::new(
    0x6074,
    0x00,
    AccessType::ReadOnly,
    MappableType::TPDO,
    ODValue::I16(0x0000),
    PdoSemantic::Other,
);

/// Torque slope — Maximum allowed change in torque per second [thousandths/s].
/// Defines the torque ramp rate.
pub const TORQUE_SLOPE: ODEntry = ODEntry::new(
    0x6087,
    0x00,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(100), // Example: 100 thousandths/s = 10% of max rated torque change per second
    PdoSemantic::Other,
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
    PdoSemantic::Other,
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
    PdoSemantic::Other,
);

/// Limit switch related
///
pub const LIMIT_SWITCH_OPTION_CODE: ODEntry = ODEntry::new(
    0x3701,
    0x00,
    AccessType::ReadWrite,
    MappableType::None,
    ODValue::I16(-1), // No reaction (e. g., to execute a homing operation) except noting the limit switch position
    PdoSemantic::Other,
);

/// Digital Input & Special Function bits
pub const DIGITAL_INPUTS: ODEntry = ODEntry::new(
    0x60FD,
    0x00,
    AccessType::ReadWrite,
    MappableType::TPDO,
    ODValue::U32(0),
    PdoSemantic::Other,
);

/// Digital input special special function control object
pub const DIGITAL_INPUTS_CONTROL_SPECIAL_FUNCTION: ODEntry = ODEntry::new(
    0x3240,
    0x01,
    AccessType::ReadOnly,
    MappableType::None,
    ODValue::U32(0),
    PdoSemantic::Other,
);

/// Whether digital input logic should be inverted [default = active low]
pub const DIGITAL_INPUTS_CONTROL_INVERTED: ODEntry = ODEntry::new(
    0x3240,
    0x02,
    AccessType::ReadOnly,
    MappableType::None,
    ODValue::U8(0),
    PdoSemantic::Other,
);

/// Contains unmodified Digital Inputs values
pub const DIGITAL_INPUTS_RAW_VALUE: ODEntry = ODEntry::new(
    0x3240,
    0x05,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0),
    PdoSemantic::Other,
);

/// Enables routing physical inputs to "digital inputs"
/// You can solve anything with a layer of indirection
pub const DIGITAL_INPUTS_ROUTING_ENABLE: ODEntry = ODEntry::new(
    0x3240,
    0x08,
    AccessType::ReadWrite,
    MappableType::RPDO,
    ODValue::U32(0),
    PdoSemantic::Other,
);

/// Determines physical source for DI 1
pub const DIGITAL_INPUTS_ROUTING_1: ODEntry = ODEntry::new(
    0x3242,
    0x01,
    AccessType::ReadOnly,
    MappableType::None,
    ODValue::U8(0),
    PdoSemantic::Other,
);

/// Determines physical source for DI 2
pub const DIGITAL_INPUTS_ROUTING_2: ODEntry = ODEntry::new(
    0x3242,
    0x02,
    AccessType::ReadOnly,
    MappableType::None,
    ODValue::U8(0),
    PdoSemantic::Other,
);

/// Determines physical source for DI 3
pub const DIGITAL_INPUTS_ROUTING_3: ODEntry = ODEntry::new(
    0x3242,
    0x03,
    AccessType::ReadOnly,
    MappableType::None,
    ODValue::U8(0),
    PdoSemantic::Other,
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

/// Minimum set of Object Dictionary entries required for Profile Torque Mode
pub const TORQUE_MODE_MINIMUM_PARAMS: &[ODEntry] = &[MAX_TORQUE, MAX_CURRENT, TORQUE_SLOPE];

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
    MAX_TORQUE,
    MAX_CURRENT,
    TORQUE_SLOPE,
];

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ODIdx {
    pub index: u16,
    pub sub_index: u8,
}

const OD_LOOKUP_SIZE: usize = 64;
static OD_LOOKUP: Lazy<FnvIndexMap<ODIdx, &ODEntry, OD_LOOKUP_SIZE>> = Lazy::new(|| {
    let mut m = FnvIndexMap::new();
    for entry in FULL_OBJECT_DICTIONARY {
        m.insert(
            ODIdx {
                index: entry.index,
                sub_index: entry.sub_index,
            },
            entry,
        )
        .expect("Unable to insert {entry:?} in OD_LOOKUP table, its likely too small; increase OD_LOOKUP_SIZE");
    }
    m
});

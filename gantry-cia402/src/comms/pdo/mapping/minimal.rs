use crate::{
    comms::pdo::mapping::{
        BitRange, PDOSet, PdoMapping, PdoMappingSource, PdoType,
        custom::{RPDO_CONTROL_OPMODE, TPDO_STATUS_OPMODE},
        empty::*,
    },
    driver::{
        oms::OperationMode, receiver::StatusWord, startup::pdo_mapping::TransmissionType,
        update::ControlWord,
    },
    od,
};

pub const MINIMAL_CYCLIC_SYNCHRONOUS_PDO_SET: PDOSet = PDOSet {
    rpdos: MINIMAL_RPODS,
    tpdos: MINIMAL_TPODS,
};

/// Minimal set of TPDO mappings for use with OnSync/Cyclic Synchronous modes
/// Having only 1 R/TPDO mapping per device minimises bus traffic
const MINIMAL_TPODS: &[PdoMapping; 4] = &[
    TPDO_STATUS_OPMODE, // Note this has transmission_type = OnChange to reduce traffic
    TPDO_STATUS_ACTUAL_POS_TORQUE,
    TPDO_EMPTY_3,
    TPDO_EMPTY_4,
];

/// Minimal set of RPDO mappings for use with OnSync/Cyclic Synchronous modes
/// Having only 1 R/TPDO mapping per device minimises bus traffic
const MINIMAL_RPODS: &[PdoMapping; 4] = &[
    RPDO_CONTROL_OPMODE, // Note this has transmission_type = OnChange to reduce traffic
    RPDO_CONTROL_TARGET_POS_TORQUE,
    RPDO_EMPTY_3,
    RPDO_EMPTY_4,
];

pub const TPDO_OPMODE: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(1),
    sources: &[PdoMappingSource {
        entry: &od::GET_OPERATION_MODE,
        bit_range: BitRange { start: 0, len: 8 },
    }],
    transmission_type: TransmissionType::OnChange,
};
pub struct Tpdo1 {
    pub opmode: OperationMode,
}

pub const TPDO_STATUS_ACTUAL_POS_TORQUE: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(2),
    sources: &[
        PdoMappingSource {
            entry: &od::STATUS_WORD,
            bit_range: BitRange { start: 0, len: 16 },
        },
        PdoMappingSource {
            entry: &od::POSITION_ACTUAL_VALUE,
            bit_range: BitRange { start: 16, len: 32 },
        },
        PdoMappingSource {
            entry: &od::TORQUE_ACTUAL_VALUE,
            bit_range: BitRange { start: 48, len: 16 },
        },
    ],
    transmission_type: TransmissionType::OnSync,
};
pub struct Tpdo2 {
    pub status: StatusWord,
    pub actual_pos: i32,
    pub actual_torque: i16,
}

pub const RPDO_CONTROL_TARGET_POS_TORQUE: PdoMapping = PdoMapping {
    pdo: PdoType::RPDO(2),
    sources: &[
        PdoMappingSource {
            entry: &od::CONTROL_WORD,
            bit_range: BitRange { start: 0, len: 16 },
        },
        PdoMappingSource {
            entry: &od::SET_TARGET_POSITION,
            bit_range: BitRange { start: 16, len: 32 },
        },
        PdoMappingSource {
            entry: &od::SET_TARGET_TORQUE,
            bit_range: BitRange { start: 48, len: 16 },
        },
    ],
    transmission_type: TransmissionType::OnSync,
};
pub struct Rpdo2 {
    control: ControlWord,
    target_pos: i32,
    target_torque: i16,
}

use crate::{
    comms::pdo::mapping::{BitRange, PDOSet, PdoMapping, PdoMappingSource, PdoType, empty::*},
    driver::startup::pdo_mapping::TransmissionType,
    od,
};

pub const CSP_PDOS: PDOSet = PDOSet {
    rpdos: CSP_RPDOS,
    tpdos: CSP_TPDOS,
};

pub const CSV_PDOS: PDOSet = PDOSet {
    rpdos: CSV_RPDOS,
    tpdos: CSV_TPDOS,
};

pub const CST_PDOS: PDOSet = PDOSet {
    rpdos: CST_RPDOS,
    tpdos: CST_TPDOS,
};

pub const CSP_TPDOS: &[PdoMapping; 4] = &[
    TPDO_CSP_STATUS_POSITION,
    TPDO_EMPTY_2,
    TPDO_EMPTY_3,
    TPDO_EMPTY_4,
];

pub const CSP_RPDOS: &[PdoMapping; 4] = &[
    RPDO_CSP_STATUS_POSITION_OPMODE,
    RPDO_EMPTY_2,
    RPDO_EMPTY_3,
    RPDO_EMPTY_4,
];

pub const CSV_TPDOS: &[PdoMapping; 4] = &[
    TPDO_CSV_STATUS_VELOCITY,
    TPDO_EMPTY_2,
    TPDO_EMPTY_3,
    TPDO_EMPTY_4,
];

pub const CSV_RPDOS: &[PdoMapping; 4] = &[
    RPDO_CSV_STATUS_VELOCITY,
    RPDO_EMPTY_2,
    RPDO_EMPTY_3,
    RPDO_EMPTY_4,
];

pub const CST_TPDOS: &[PdoMapping; 4] = &[
    TPDO_CST_STATUS_TORQUE,
    TPDO_EMPTY_2,
    TPDO_EMPTY_3,
    TPDO_EMPTY_4,
];

pub const CST_RPDOS: &[PdoMapping; 4] = &[
    RPDO_CST_STATUS_TORQUE,
    RPDO_EMPTY_2,
    RPDO_EMPTY_3,
    RPDO_EMPTY_4,
];

pub const RPDO_CSP_STATUS_POSITION_OPMODE: PdoMapping = PdoMapping {
    pdo: PdoType::RPDO(1),
    sources: &[
        PdoMappingSource {
            entry: &od::CONTROL_WORD,
            bit_range: BitRange { start: 0, len: 16 },
        },
        PdoMappingSource {
            entry: &od::SET_OPERATION_MODE,
            bit_range: BitRange { start: 16, len: 8 },
        },
        PdoMappingSource {
            entry: &od::SET_TARGET_POSITION,
            bit_range: BitRange { start: 24, len: 32 },
        },
    ],
    transmission_type: TransmissionType::OnSync,
};

pub const TPDO_CSP_STATUS_POSITION: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(1),
    sources: &[
        PdoMappingSource {
            entry: &od::STATUS_WORD,
            bit_range: BitRange { start: 0, len: 16 },
        },
        PdoMappingSource {
            entry: &od::POSITION_ACTUAL_VALUE,
            bit_range: BitRange { start: 16, len: 32 },
        },
    ],
    transmission_type: TransmissionType::OnSync,
};

pub const RPDO_CSV_STATUS_VELOCITY: PdoMapping = PdoMapping {
    pdo: PdoType::RPDO(1),
    sources: &[
        PdoMappingSource {
            entry: &od::CONTROL_WORD,
            bit_range: BitRange { start: 0, len: 16 },
        },
        PdoMappingSource {
            entry: &od::SET_OPERATION_MODE,
            bit_range: BitRange { start: 16, len: 8 },
        },
        PdoMappingSource {
            entry: &od::SET_TARGET_VELOCITY,
            bit_range: BitRange { start: 24, len: 32 },
        },
    ],
    transmission_type: TransmissionType::OnSync,
};

pub const TPDO_CSV_STATUS_VELOCITY: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(1),
    sources: &[
        PdoMappingSource {
            entry: &od::STATUS_WORD,
            bit_range: BitRange { start: 0, len: 16 },
        },
        PdoMappingSource {
            entry: &od::VELOCITY_ACTUAL_VALUE,
            bit_range: BitRange { start: 16, len: 32 },
        },
    ],
    transmission_type: TransmissionType::OnSync,
};

pub const RPDO_CST_STATUS_TORQUE: PdoMapping = PdoMapping {
    pdo: PdoType::RPDO(1),
    sources: &[
        PdoMappingSource {
            entry: &od::CONTROL_WORD,
            bit_range: BitRange { start: 0, len: 16 },
        },
        PdoMappingSource {
            entry: &od::SET_OPERATION_MODE,
            bit_range: BitRange { start: 16, len: 8 },
        },
        PdoMappingSource {
            entry: &od::SET_TARGET_TORQUE,
            bit_range: BitRange { start: 24, len: 16 },
        },
    ],
    transmission_type: TransmissionType::OnSync,
};

pub const TPDO_CST_STATUS_TORQUE: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(1),
    sources: &[
        PdoMappingSource {
            entry: &od::STATUS_WORD,
            bit_range: BitRange { start: 0, len: 16 },
        },
        PdoMappingSource {
            entry: &od::TORQUE_ACTUAL_VALUE,
            bit_range: BitRange { start: 16, len: 16 },
        },
    ],
    transmission_type: TransmissionType::OnSync,
};

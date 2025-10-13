use crate::{
    comms::pdo::mapping::{BitRange, PdoMapping, PdoMappingSource, PdoType},
    driver::startup::pdo_mapping::TransmissionType,
    od,
};

// TODO: Making T/RPDO mapping generic + encoding into type system
// I didn't know of a better const method to do this, seems rust const fn are lacking compared
// to c++ templates, this never changes anyway
pub const RPDO_IDX_CONTROL_WORD: usize = 0;
pub const RPDO_IDX_OPMODE: usize = 0;
pub const RPDO_IDX_TARGET_POS: usize = 1;
pub const RPDO_IDX_TARGET_VEL: usize = 2;
pub const RPDO_IDX_TARGET_TORQUE: usize = 3;

pub const CUSTOM_RPDOS: &[PdoMapping; 4] = &[
    RPDO_CONTROL_OPMODE,
    RPDO_TARGET_POS,
    RPDO_TARGET_VEL,
    RPDO_TARGET_TORQUE,
];

pub const CUSTOM_TPDOS: &[PdoMapping; 4] = &[
    TPDO_STATUS_OPMODE,
    TPDO_POS_VEL_ACTUAL,
    TPDO_TORQUE_ACTUAL,
    TPDO_EMPTY, // Required to avoid default TPDO4 generating warnings, TODO: remove this when
                // adding invalidate all PDO step in configure_pdo_mappings
];

pub fn get_dlc(mapping: &PdoMapping) -> usize {
    let mut dlc = 0u8;
    for source in mapping.sources {
        dlc += source.bit_range.len / 8;
    }

    assert!(dlc <= 8);
    dlc as usize
}

pub const RPDO_CONTROL_OPMODE: PdoMapping = PdoMapping {
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
    ],
    transmission_type: TransmissionType::OnChange,
};

pub const RPDO_TARGET_POS: PdoMapping = PdoMapping {
    pdo: PdoType::RPDO(2),
    sources: &[
        PdoMappingSource {
            entry: &od::SET_TARGET_POSITION,
            bit_range: BitRange { start: 0, len: 32 },
        },
        PdoMappingSource {
            entry: &od::PROFILE_VELOCITY,
            bit_range: BitRange { start: 32, len: 32 },
        },
    ],
    transmission_type: TransmissionType::OnChange,
};

pub const RPDO_TARGET_VEL: PdoMapping = PdoMapping {
    pdo: PdoType::RPDO(3),
    sources: &[PdoMappingSource {
        entry: &od::SET_TARGET_VELOCITY,
        bit_range: BitRange { start: 0, len: 32 },
    }],
    transmission_type: TransmissionType::OnChange,
};

pub const RPDO_TARGET_TORQUE: PdoMapping = PdoMapping {
    pdo: PdoType::RPDO(4),
    sources: &[PdoMappingSource {
        entry: &od::SET_TARGET_TORQUE,
        bit_range: BitRange { start: 0, len: 16 },
    }],
    transmission_type: TransmissionType::OnChange,
};

pub const TPDO_STATUS_OPMODE: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(1),
    sources: &[
        PdoMappingSource {
            entry: &od::STATUS_WORD,
            bit_range: BitRange { start: 0, len: 16 },
        },
        PdoMappingSource {
            entry: &od::GET_OPERATION_MODE,
            bit_range: BitRange { start: 16, len: 8 },
        },
    ],
    transmission_type: TransmissionType::OnChange,
};

pub const TPDO_POS_VEL_ACTUAL: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(2),
    sources: &[
        PdoMappingSource {
            entry: &od::POSITION_ACTUAL_VALUE,
            bit_range: BitRange { start: 0, len: 32 },
        },
        PdoMappingSource {
            entry: &od::VELOCITY_ACTUAL_VALUE,
            bit_range: BitRange { start: 32, len: 32 },
        },
    ],
    transmission_type: TransmissionType::OnChange,
};

pub const TPDO_TORQUE_ACTUAL: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(3),
    sources: &[PdoMappingSource {
        entry: &od::TORQUE_ACTUAL_VALUE,
        bit_range: BitRange { start: 0, len: 16 },
    }],
    transmission_type: TransmissionType::OnChange,
};

pub const TPDO_EMPTY: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(4),
    sources: &[],
    transmission_type: TransmissionType::OnChange,
};

use crate::{
    comms::pdo::mapping::{BitRange, PdoMapping, PdoMappingSource, PdoType},
    od,
};

pub const DEFAULT_RPDOS: &[PdoMapping] = &[RPDO_DEFAULT_1, RPDO_DEFAULT_2];
pub const DEFAULT_TPDOS: &[PdoMapping] = &[TPDO_DEFAULT_1, TPDO_DEFAULT_2];

pub const RPDO_DEFAULT_1: PdoMapping = PdoMapping {
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
};

pub const RPDO_DEFAULT_2: PdoMapping = PdoMapping {
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
};

pub const TPDO_DEFAULT_1: PdoMapping = PdoMapping {
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
};

pub const TPDO_DEFAULT_2: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(2),
    sources: &[PdoMappingSource {
        entry: &od::POSITION_ACTUAL_VALUE,
        bit_range: BitRange { start: 0, len: 32 },
    }],
};

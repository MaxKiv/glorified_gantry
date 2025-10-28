use crate::{
    comms::pdo::mapping::{
        BitRange, PDOSet, PdoMapping, PdoMappingSource, PdoType, custom::*, empty::*,
    },
    driver::startup::pdo_mapping::TransmissionType,
    od,
};

pub const TEST_PDOS: PDOSet = PDOSet {
    rpdos: CUSTOM_RPDOS,
    tpdos: TEST_TPDOS,
};

pub const TEST_TPDOS: &[PdoMapping; 4] = &[
    TPDO_STATUS_OPMODE,
    TPDO_POS_VEL_ACTUAL,
    TPDO_TORQUE_ACTUAL, // Causes a lot of bus spam if transmission_type = OnChange, reduced by inhibit time
    TPDO_IO, // Required to avoid default TPDO4 generating warnings, TODO: remove this when
             // adding invalidate all PDO step in configure_pdo_mappings
];

pub const TPDO_IO: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(2),
    sources: &[PdoMappingSource {
        entry: &od::DIGITAL_INPUTS,
        bit_range: BitRange { start: 0, len: 32 },
    }],
    transmission_type: TransmissionType::OnChange,
};

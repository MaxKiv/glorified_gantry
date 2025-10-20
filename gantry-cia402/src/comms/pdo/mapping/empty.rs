use crate::{
    comms::pdo::mapping::{BitRange, PdoMapping, PdoMappingSource, PdoType},
    driver::startup::pdo_mapping::TransmissionType,
    od,
};

pub const TPDO_EMPTY_1: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(1),
    sources: &[],
    transmission_type: TransmissionType::OnChange,
};

pub const TPDO_EMPTY_2: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(2),
    sources: &[],
    transmission_type: TransmissionType::OnChange,
};

pub const TPDO_EMPTY_3: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(3),
    sources: &[],
    transmission_type: TransmissionType::OnChange,
};

pub const TPDO_EMPTY_4: PdoMapping = PdoMapping {
    pdo: PdoType::TPDO(4),
    sources: &[],
    transmission_type: TransmissionType::OnChange,
};

pub const RPDO_EMPTY_1: PdoMapping = PdoMapping {
    pdo: PdoType::RPDO(1),
    sources: &[],
    transmission_type: TransmissionType::OnChange,
};

pub const RPDO_EMPTY_2: PdoMapping = PdoMapping {
    pdo: PdoType::RPDO(2),
    sources: &[],
    transmission_type: TransmissionType::OnChange,
};

pub const RPDO_EMPTY_3: PdoMapping = PdoMapping {
    pdo: PdoType::RPDO(3),
    sources: &[],
    transmission_type: TransmissionType::OnChange,
};

pub const RPDO_EMPTY_4: PdoMapping = PdoMapping {
    pdo: PdoType::RPDO(4),
    sources: &[],
    transmission_type: TransmissionType::OnChange,
};

use crate::canopen::{
    od::entry::ODEntry,
    pdo::{PdoType, TransmissionType},
};

#[derive(Debug, PartialEq, Eq)]
/// Represents a single T/RPDO mapping
pub struct PdoMapping {
    // PDO type and number
    pub pdo: PdoType,
    // Values to map
    pub sources: &'static [PdoMappingSource],
    // When to transmit this PDO
    pub transmission_type: TransmissionType,
}

#[derive(Debug, PartialEq, Eq)]
/// Values to map onto T/RPDO
pub struct PdoMappingSource {
    // The entry to map
    pub entry: &'static ODEntry,
    // Start of bitrange of PDO mapping
    pub start: u8,
    // Length of bitrange of PDO mapping
    pub len: u8,
}

impl PdoMappingSource {
    pub const fn from_od_entry(od_entry: &'static ODEntry, start: u8) -> Self {
        PdoMappingSource {
            entry: &od_entry,
            start,
            len: od_entry.get_num_bits(),
        }
    }
}

/// Per node pdo mapping
#[derive(Default)]
pub struct OMSNodePdoConfig {
    pub tpdo1: Option<PdoMapping>,
    pub tpdo2: Option<PdoMapping>,
    pub tpdo3: Option<PdoMapping>,
    pub tpdo4: Option<PdoMapping>,
    pub rpdo1: Option<PdoMapping>,
    pub rpdo2: Option<PdoMapping>,
    pub rpdo3: Option<PdoMapping>,
    pub rpdo4: Option<PdoMapping>,
}

impl OMSNodePdoConfig {
    pub const fn empty() -> Self {
        OMSNodePdoConfig {
            tpdo1: None,
            tpdo2: None,
            tpdo3: None,
            tpdo4: None,
            rpdo1: None,
            rpdo2: None,
            rpdo3: None,
            rpdo4: None,
        }
    }
}

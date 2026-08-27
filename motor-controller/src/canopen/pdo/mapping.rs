use crate::canopen::{
    od::{
        entry::{ODEntry, PdoSemantic},
        value::ODValue,
    },
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

impl PdoMapping {
    // Encodes the given values into the given buffer, ready to be sent over the wire
    // NOTE: this opportunistically encodes values, it is the users responsibility to
    // provide &[`PdoValue`] with the same order and semantic meaning as defined in this mapping
    pub fn encode(&self, values: &[PdoValue], data: &mut [u8; 8]) -> Result<(), ()> {
        for (i, src) in self.sources.iter().enumerate() {
            // TODO: encode values[i]; -> LE(!) bits
        }

        Ok(())
    }
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

/// Operation Mode specific pdo mapping for a single node
#[derive(Default)]
pub struct OMSNodePdoConfig {
    pub tpdo: [Option<PdoMapping>; 4],
    pub rpdo: [Option<PdoMapping>; 4],
}

#[derive(Debug, PartialEq, Eq)]
pub struct PdoValue {
    pub semantic: PdoSemantic,
    pub value: ODValue,
}

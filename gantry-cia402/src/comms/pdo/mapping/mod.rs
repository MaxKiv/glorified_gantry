pub mod custom;
pub mod default;

use crate::driver::startup::pdo_mapping::TransmissionType;
use crate::od::entry::ODEntry;

#[derive(Debug, Clone, Copy)]
pub struct BitRange {
    pub start: u8,
    pub len: u8,
}

#[derive(Debug, Clone)]
pub enum PdoType {
    RPDO(u8),
    TPDO(u8),
}

impl PdoType {
    pub fn to_string(&self) -> String {
        match self {
            PdoType::RPDO(num) => format!("RPDO{num}"),
            PdoType::TPDO(num) => format!("TPDO{num}"),
        }
    }
}

#[derive(Debug)]
/// Represents a single T/RPDO mapping
pub struct PdoMapping {
    // PDO type and number
    pub pdo: PdoType,
    // Values to map
    pub sources: &'static [PdoMappingSource],
    // When to transmit this PDO
    pub transmission_type: TransmissionType,
}

#[derive(Debug)]
/// Values to map onto T/RPDO
pub struct PdoMappingSource {
    // The entry to map
    pub entry: &'static ODEntry,

    // The T/RPDO bits to map the above entry to
    pub bit_range: BitRange,
}

impl PdoType {
    /// Returns the COB Id for the given pdo num and type
    /// See https://en.wikipedia.org/wiki/CANopen#Process_Data_Object_(PDO)_protocol
    pub fn get_pdo_cob_id(&self, node_id: u8) -> Option<u16> {
        Some(match self {
            Self::TPDO(num) => {
                const BASE: u16 = 0x80;
                let num = *num as u16;
                let node_id = node_id as u16;
                BASE + (0x100 * num) + node_id
            }
            Self::RPDO(num) => {
                const BASE: u16 = 0x100;
                let num = *num as u16;
                let node_id = node_id as u16;
                BASE + (0x100 * num) + node_id
            }
        })
    }
}

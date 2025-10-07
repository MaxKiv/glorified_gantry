pub mod custom;
pub mod default;

use crate::od;
use crate::od::entry::ODEntry;
use crate::od::mappable::MappableType::RPDO;
use crate::od::mappable::MappableType::TPDO;

#[derive(Debug, Clone, Copy)]
pub struct BitRange {
    pub start: u8,
    pub len: u8,
}

#[derive(Debug)]
pub enum PdoType {
    RPDO(u8),
    TPDO(u8),
}

#[derive(Debug)]
/// Represents a single T/RPDO mapping
pub struct PdoMapping {
    // PDO type and number
    pub pdo: PdoType,
    // Values to map
    pub sources: &'static [PdoMappingSource],
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
    pub fn get_pdo_cob_id(&self) -> Option<u16> {
        Some(match self {
            Self::RPDO(num) if *num == 1 => 0x180,
            Self::RPDO(num) if *num == 2 => 0x280,
            Self::RPDO(num) if *num == 3 => 0x380,
            Self::RPDO(num) if *num == 4 => 0x480,
            Self::TPDO(num) if *num == 1 => 0x180 + 0x20,
            Self::TPDO(num) if *num == 2 => 0x280 + 0x20,
            Self::TPDO(num) if *num == 3 => 0x380 + 0x20,
            Self::TPDO(num) if *num == 4 => 0x480 + 0x20,
            _ => return None,
        })
    }
}

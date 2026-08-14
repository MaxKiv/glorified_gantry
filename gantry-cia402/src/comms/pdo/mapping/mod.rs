pub mod cyclic_synchronous;
pub mod default;
pub mod empty;
pub mod minimal;
pub mod table;
pub mod test;

use crate::comms::pdo::mapping::default::RPDO_CONTROL_OPMODE;
use crate::comms::pdo::mapping::minimal::RPDO_CONTROL_TARGET_POS_TORQUE;
use crate::driver::startup::pdo_mapping::TransmissionType;
use crate::od::entry::ODEntry;

#[derive(Debug)]
pub struct PdoSet {
    pub rpdos: &'static [PdoMapping; 4],
    pub tpdos: &'static [PdoMapping; 4],
}

impl PdoSet {
    /// Checks if the RPDO_CONTROL_OPMODE PdoMapping is contained within the given default pdo set
    /// Shitty method of guarding a future maintainer against my rushed design
    /// TODO: Move this into type system, improve PDO parsing in general
    pub fn contains_default_rpdo(&self) -> bool {
        const REQUIRED_RPDO: PdoMapping = RPDO_CONTROL_OPMODE;

        self.rpdos
            .iter()
            .find(|map| **map == REQUIRED_RPDO)
            .is_some()
    }

    /// Checks if the RPDO_CONTROL_TARGET_POS_TORQUE PdoMapping is contained within the given minimal pdo set
    /// Shitty method of guarding a future maintainer against my rushed design
    /// TODO: Move this into type system, improve PDO parsing in general
    pub fn contains_minimal_rpdo(&self) -> bool {
        const REQUIRED_RPDO: PdoMapping = RPDO_CONTROL_TARGET_POS_TORQUE;

        self.rpdos
            .iter()
            .find(|map| **map == REQUIRED_RPDO)
            .is_some()
    }

    pub fn get_mapping_source_for_od_entry(
        &self,
        entry: &'static ODEntry,
    ) -> Option<(usize, &'static PdoMappingSource)> {
        for (pdo_num, mapping) in self.rpdos.iter().enumerate() {
            for source in mapping.sources.iter() {
                if source.entry == entry {
                    return Some((pdo_num, source));
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitRange {
    pub start: u8,
    pub len: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PdoType {
    RPDO(u8),
    TPDO(u8),
}

impl PdoType {
    pub fn to_string_pretty(&self) -> String {
        match self {
            PdoType::RPDO(num) => format!("RPDO{num}"),
            PdoType::TPDO(num) => format!("TPDO{num}"),
        }
    }
}

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

    // The T/RPDO bits to map the above entry to
    pub bit_range: BitRange,
}

impl PdoMapping {
    pub fn get_dlc(&self) -> usize {
        let mut dlc = 0u8;
        for source in self.sources {
            dlc += source.bit_range.len / 8;
        }

        dlc as usize
    }
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

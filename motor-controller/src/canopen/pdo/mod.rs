pub mod frame;
pub mod mapping;
pub mod message;

/// Binary representation of a RPDO
pub type RawRpdo = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdoType {
    TPDO,
    RPDO,
}

impl PdoType {
    /// Returns the COB Id for the given pdo num and type
    /// See https://en.wikipedia.org/wiki/CANopen#Process_Data_Object_(PDO)_protocol
    pub fn get_pdo_cob_id(&self, num: u16, node_id: u8) -> Option<u16> {
        Some(match self {
            Self::TPDO => {
                const BASE: u16 = 0x80;
                let node_id = node_id as u16;
                BASE + (0x100 * num) + node_id
            }
            Self::RPDO => {
                const BASE: u16 = 0x100;
                let node_id = node_id as u16;
                BASE + (0x100 * num) + node_id
            }
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
/// Send TPDO on every Nth sync
pub struct OnSyncN(u8);

impl OnSyncN {
    pub const fn from(from: u8) -> Option<Self> {
        // Datasheet 8.2.2 & page 120 says maximum of 240, 0 is reserved for RPDO
        if from <= 240 && from > 0 {
            Some(Self(from))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransmissionType {
    OnSyncRPDO,
    OnSyncTPDO(OnSyncN),
    OnChange,
}

impl TransmissionType {
    pub fn od_value(&self) -> u8 {
        match self {
            TransmissionType::OnSyncRPDO => 0x0,
            TransmissionType::OnChange => 0xFF,
            TransmissionType::OnSyncTPDO(n) => n.0,
        }
    }
}

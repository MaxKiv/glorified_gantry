pub mod mapping;
pub mod message;

#[derive(Debug, PartialEq, Eq)]
pub enum PdoType {
    TPDO,
    RPDO,
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

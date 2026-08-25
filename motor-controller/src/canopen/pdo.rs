use socketcan::{CanDataFrame, EmbeddedFrame};

use crate::canopen::frame::{CanOpenParseError, CobId, NodeId};

#[derive(Debug)]
pub struct RawPdoMessage {
    pub node_id: NodeId,
    pub num: usize,
    pub pdo_type: PdoType,
    pub data: [u8; 8],
    pub dlc: usize,
}

impl RawPdoMessage {
    pub fn try_from_can_frame(
        cob_id: CobId,
        frame: CanDataFrame,
    ) -> Result<Self, CanOpenParseError> {
        let (pdo_type, num, base) = match cob_id.0 {
            0x180..=0x1FF => (PdoType::TPDO, 1, 0x180),
            0x200..=0x21F => (PdoType::RPDO, 1, 0x200),
            0x280..=0x2FF => (PdoType::TPDO, 2, 0x280),
            0x300..=0x31F => (PdoType::RPDO, 2, 0x300),
            0x380..=0x3FF => (PdoType::TPDO, 3, 0x380),
            0x400..=0x41F => (PdoType::RPDO, 3, 0x400),
            0x480..=0x4FF => (PdoType::TPDO, 4, 0x480),
            0x500..=0x51F => (PdoType::RPDO, 4, 0x500),
            _ => {
                return Err(CanOpenParseError::PdoNumRange(frame));
            }
        };
        let node = NodeId((cob_id.0 - base) as u8);
        let dlc = frame.dlc();

        let msg = RawPdoMessage {
            node_id: node,
            num,
            pdo_type,
            data: frame
                .data()
                .try_into()
                .map_err(|_| CanOpenParseError::WrongDLC(dlc))?,
            dlc,
        };

        Ok(msg)
    }
}

#[derive(Debug)]
pub enum PdoType {
    TPDO,
    RPDO,
}

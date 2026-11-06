use socketcan::{CanFrame, Frame};

use crate::{
    canopen::pdo::{PdoType, RawRpdo},
    cia402::Cia402Identifier,
};

#[derive(Debug)]
pub struct RpdoFrame {
    pub inner: CanFrame,
}

impl RpdoFrame {
    pub fn new_from_raw_rpdo(raw_rpdo: RawRpdo, num: u16, node: &Cia402Identifier) -> Self {
        let cob_id = PdoType::RPDO
            .get_pdo_cob_id(node.id)
            .ok_or(DriveError::ViolatedInvariant(
                "Asked for the cob_id for PDO number: {rpdo_num} > 4".to_string(),
            ))?;

        let inner: CanFrame = CanFrame::from_raw_id(cob_id, &raw_rpdo.to_le_bytes())
            .expect("failed to construct NMT frame");

        Self { inner }
    }
}

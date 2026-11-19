use socketcan::{CanFrame, Frame};

use crate::canopen::{
    CanOpenError,
    frame::NodeId,
    pdo::{PdoType, RawRpdo},
};

#[derive(Debug)]
pub struct RpdoFrame {
    pub inner: CanFrame,
}

impl RpdoFrame {
    pub fn new_from_raw_rpdo(
        raw_rpdo: RawRpdo,
        num: u16,
        id: NodeId,
    ) -> Result<Self, CanOpenError> {
        let cob_id = PdoType::RPDO
            .get_pdo_cob_id(num, id.u8())
            .ok_or(CanOpenError::RpdoFrame(raw_rpdo))?;

        let inner: CanFrame = CanFrame::from_raw_id(cob_id as u32, &raw_rpdo.to_le_bytes())
            .ok_or(CanOpenError::RpdoFrame(raw_rpdo))?;

        Ok(Self { inner })
    }
}

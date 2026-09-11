use crate::{
    canopen::sdo::{SdoDownload, SdoUpload},
    cia402::Cia402Identifier,
};

use socketcan::{CanFrame, Frame};

const SDO_REQUEST_BASE_COB_ID: u32 = 0x600;
const SDO_RESPONSE_BASE_COB_ID: u32 = 0x580;

/// SDO client command specifiers (byte 0 of a request frame).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SdoCommandSpecifier {
    /// Expedited download (write), size indicated.
    /// Value encodes ccs = 0b001 with e=1, s=1.
    Download4 = 0x23,
    Download3 = 0x27,
    Download2 = 0x2B,
    Download1 = 0x2F,
    /// Initiate domain upload (read). ccs = 0b010.
    Upload = 0x40,
    /// Abort segment transfer.
    Abort = 0x80,
}

impl SdoCommandSpecifier {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Pick the download command matching the data length (1–4 bytes).
    pub fn expedited_download(len: usize) -> Option<Self> {
        match len {
            1 => Some(Self::Download1),
            2 => Some(Self::Download2),
            3 => Some(Self::Download3),
            4 => Some(Self::Download4),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct SdoFrame {
    pub inner: CanFrame,
}

impl SdoFrame {
    /// Build an expedited SDO download (write) request, 1–4 data bytes
    pub fn new_write(sdo: SdoDownload) -> Self {
        let mut data = [0u8; 8];
        let len = sdo.value.to_le_bytes(&mut data);
        let cmd = SdoCommandSpecifier::expedited_download(len)
            .expect("SDO expedited download requires 1-4 data bytes");

        Self::new_raw_request(
            sdo.node,
            cmd,
            sdo.od_entry.index,
            sdo.od_entry.sub_index,
            &data,
        )
    }

    /// Build an SDO upload (read) request
    pub fn new_read(sdo: SdoUpload) -> Self {
        Self::new_raw_request(
            sdo.node,
            SdoCommandSpecifier::Upload,
            sdo.od_entry.index,
            sdo.od_entry.sub_index,
            &[],
        )
    }

    /// Wrap a received CAN frame (from 0x580 + node_id) into an SdoFrame response
    pub fn from_response(frame: CanFrame) -> Self {
        debug_assert!(
            (frame.id_word() & !0x7F) == SDO_RESPONSE_BASE_COB_ID,
            "CAN frame is not an SDO response (expected COB-ID 0x580 + node_id)"
        );
        Self { inner: frame }
    }

    fn new_raw_request(
        node: &Cia402Identifier,
        cmd: SdoCommandSpecifier,
        index: u16,
        sub_index: u8,
        data: &[u8],
    ) -> Self {
        let cob_id = SDO_REQUEST_BASE_COB_ID + u32::from(node.node_id.u8());

        let mut payload = [0u8; 8];
        payload[0] = cmd.as_u8();
        payload[1] = index as u8;
        payload[2] = (index >> 8) as u8;
        payload[3] = sub_index;
        payload[4..4 + data.len()].copy_from_slice(data);

        let inner: CanFrame =
            CanFrame::from_raw_id(cob_id, &payload).expect("failed to construct SDO frame");

        Self { inner }
    }
}

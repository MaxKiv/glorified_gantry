use socketcan::{CanDataFrame, EmbeddedFrame};

use crate::canopen::{
    frame::{CanOpenParseError, CobId, NodeId},
    od::entry::ODEntry,
};

#[derive(Debug)]
pub struct SdoRequest {
    pub data: [u8; 8],
    pub dlc: usize,
    pub value: Option<ODEntry>,
}

impl SdoRequest {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SdoError {
    pub from: NodeId,
    pub index: u16,
    pub sub_index: u8,
    pub code: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SdoUploadResult {
    pub from: NodeId,
    pub dlc: u8,
    pub index: u16,
    pub sub_index: u8,
    pub data: [u8; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SdoDownloadConfirmed {
    pub from: NodeId,
    pub index: u16,
    pub sub_index: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdoResponse {
    Error(SdoError),
    DownloadConfirm(SdoDownloadConfirmed),
    UploadConfirm(SdoUploadResult),
}

impl SdoResponse {
    pub fn try_from_frame(cob_id: CobId, frame: &CanDataFrame) -> Result<Self, CanOpenParseError> {
        let from = NodeId::new((cob_id.0 - 0x580) as u8);
        let dlc = frame.dlc();
        let data = frame.data();
        let payload = &data[4..dlc.min(8)];

        match data[0] {
            0x80 => Ok(SdoResponse::Error(SdoError {
                from,
                index: u16::from_le_bytes(
                    data[1..3]
                        .try_into()
                        .map_err(|_| CanOpenParseError::SdoInvalidData(frame.clone()))?,
                ),
                sub_index: data[3],
                code: u32::from_le_bytes(
                    payload
                        .try_into()
                        .map_err(|_| CanOpenParseError::SdoInvalidData(frame.clone()))?,
                ),
            })),
            0x60 => Ok(SdoResponse::DownloadConfirm(SdoDownloadConfirmed {
                from,
                index: u16::from_le_bytes(
                    data[1..3]
                        .try_into()
                        .map_err(|_| CanOpenParseError::SdoInvalidData(frame.clone()))?,
                ),
                sub_index: data[3],
            })),
            0x4F => Ok(SdoResponse::UploadConfirm(SdoUploadResult {
                from,
                dlc: 1,
                index: u16::from_le_bytes(
                    data[1..3]
                        .try_into()
                        .map_err(|_| CanOpenParseError::SdoInvalidData(frame.clone()))?,
                ),
                sub_index: data[3],
                data: payload
                    .try_into()
                    .map_err(|_| CanOpenParseError::SdoInvalidData(frame.clone()))?,
            })),
            0x4B => Ok(SdoResponse::UploadConfirm(SdoUploadResult {
                from,
                dlc: 2,
                index: u16::from_le_bytes(
                    data[1..3]
                        .try_into()
                        .map_err(|_| CanOpenParseError::SdoInvalidData(frame.clone()))?,
                ),
                sub_index: data[3],
                data: payload
                    .try_into()
                    .map_err(|_| CanOpenParseError::SdoInvalidData(frame.clone()))?,
            })),
            0x47 => Ok(SdoResponse::UploadConfirm(SdoUploadResult {
                from,
                dlc: 3,
                index: u16::from_le_bytes(
                    data[1..3]
                        .try_into()
                        .map_err(|_| CanOpenParseError::SdoInvalidData(frame.clone()))?,
                ),
                sub_index: data[3],
                data: payload
                    .try_into()
                    .map_err(|_| CanOpenParseError::SdoInvalidData(frame.clone()))?,
            })),
            0x43 => Ok(SdoResponse::UploadConfirm(SdoUploadResult {
                from,
                dlc: 4,
                index: u16::from_le_bytes(
                    data[1..3]
                        .try_into()
                        .map_err(|_| CanOpenParseError::SdoInvalidData(frame.clone()))?,
                ),
                sub_index: data[3],
                data: payload
                    .try_into()
                    .map_err(|_| CanOpenParseError::SdoInvalidData(frame.clone()))?,
            })),
            _ => {
                return Err(CanOpenParseError::SdoInvalidData(frame.clone()));
            }
        }
    }

    pub fn fmt_pretty(&self) -> String {
        match &self {
            SdoResponse::Error(sdo_error) => format!(
                "SDO Error for {:#0x}:{} - code {:#0x}",
                sdo_error.index, sdo_error.sub_index, sdo_error.code
            )
            .to_string(),
            SdoResponse::DownloadConfirm(sdo_download_result) => format!(
                "SDO Download Confirm for {:#0x}:{}",
                sdo_download_result.index, sdo_download_result.sub_index
            )
            .to_string(),
            SdoResponse::UploadConfirm(sdo_upload_result) => format!(
                "SDO Upload Confirm for {:#0x}:{} => [{}]",
                sdo_upload_result.index,
                sdo_upload_result.sub_index,
                hex_dump(&sdo_upload_result.data)
            )
            .to_string(),
        }
    }
}

pub fn hex_dump(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:#2X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

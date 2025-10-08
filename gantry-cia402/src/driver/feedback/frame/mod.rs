pub mod log;
pub mod parse;

use oze_canopen::{
    canopen::{NodeId, RxMessage},
    proto::sdo,
};
use thiserror::Error;
use tokio::time::Instant;
use tracing::*;

use crate::{
    comms::pdo::mapping::PdoType,
    driver::{feedback::frame::log::hex_dump, nmt::NmtState},
    od::{entry::ODEntry, value::ODValue},
};

#[derive(Debug)]
pub struct ParseError(anyhow::Error);

pub enum Frame {
    NmtControl(NmtControlMessage),
    Sync(SyncMessage),
    EMCY(EmergencyMessage),
    TSDO(SdoResponse),
    RSDO(SdoRequest),
    TPDO(TPDOMessage),
    RPDO(RPDOMessage),
    NmtMonitor(NmtMonitorMessage),
    Unknown(RxMessage),
}

pub struct NmtControlMessage {
    timestamp: Instant,
    requested_state: NmtState,
    to: NodeId,
}

pub struct NmtMonitorMessage {
    timestamp: Instant,
    from: NodeId,
    current_state: NmtState,
}

pub struct SyncMessage {
    timestamp: Instant,
}

#[derive(Debug)]
pub struct EmergencyMessage {
    timestamp: Instant,
    from: NodeId,
    error: EMCY,
}

#[derive(Debug)]
pub enum EMCY {
    Undervoltage,
    Unknown,
}

pub struct TPDOMessage {
    timestamp: Instant,
    from: NodeId,
    num: usize,
    data: [u8; 8],
    dlc: usize,
}

pub struct RPDOMessage {
    timestamp: Instant,
    from: NodeId,
    num: usize,
    data: [u8; 8],
    dlc: usize,
}

pub struct SdoMessage {
    timestamp: Instant,
    from: NodeId,
    data: [u8; 8],
    dlc: usize,
    value: Option<ODEntry>,
}

#[derive(Debug)]
pub struct SdoRequest {
    timestamp: Instant,
    from: NodeId,
    data: [u8; 8],
    dlc: usize,
    value: Option<ODEntry>,
}

#[derive(Debug)]
pub struct SdoError {
    from: NodeId,
    index: u16,
    sub_index: u8,
    code: u32,
}

#[derive(Debug)]
pub struct SdoUploadResult {
    from: NodeId,
    dlc: u8,
    index: u16,
    sub_index: u8,
    data: [u8; 4],
}

#[derive(Debug)]
pub struct SdoDownloadConfirmed {
    from: NodeId,
    index: u16,
    sub_index: u8,
}

#[derive(Debug)]
pub enum SdoResponse {
    Error(SdoError),
    DownloadConfirm(SdoDownloadConfirmed),
    UploadConfirm(SdoUploadResult),
}

impl SdoResponse {
    pub fn from_frame(frame: &RxMessage) -> anyhow::Result<Self> {
        let from = (frame.cob_id - 0x580) as u8;
        let payload = &frame.data[4..frame.dlc.min(8)];

        match frame.data[0] {
            0x80 => Ok(SdoResponse::Error(SdoError {
                from,
                index: u16::from_le_bytes(frame.data[1..3].try_into()?),
                sub_index: frame.data[3],
                code: u32::from_le_bytes(payload.try_into()?),
            })),
            0x60 => Ok(SdoResponse::DownloadConfirm(SdoDownloadConfirmed {
                from,
                index: u16::from_le_bytes(frame.data[1..3].try_into()?),
                sub_index: frame.data[3],
            })),
            0x4F => Ok(SdoResponse::UploadConfirm(SdoUploadResult {
                from,
                dlc: 1,
                index: u16::from_le_bytes(frame.data[1..3].try_into()?),
                sub_index: frame.data[3],
                data: payload.try_into()?,
            })),
            0x4B => Ok(SdoResponse::UploadConfirm(SdoUploadResult {
                from,
                dlc: 2,
                index: u16::from_le_bytes(frame.data[1..3].try_into()?),
                sub_index: frame.data[3],
                data: payload.try_into()?,
            })),
            0x47 => Ok(SdoResponse::UploadConfirm(SdoUploadResult {
                from,
                dlc: 3,
                index: u16::from_le_bytes(frame.data[1..3].try_into()?),
                sub_index: frame.data[3],
                data: payload.try_into()?,
            })),
            0x43 => Ok(SdoResponse::UploadConfirm(SdoUploadResult {
                from,
                dlc: 4,
                index: u16::from_le_bytes(frame.data[1..3].try_into()?),
                sub_index: frame.data[3],
                data: payload.try_into()?,
            })),
            _ => {
                anyhow::bail!("Unable to parse {frame:?} into SdoResponse");
            }
        }
    }

    pub fn fmt_pretty(&self) -> String {
        match &self {
            SdoResponse::Error(sdo_error) => format!(
                "SDO Error for {:#0x}:{}- code {:#0x}",
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

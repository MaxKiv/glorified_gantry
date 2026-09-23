pub mod emcy;
pub mod frame;
pub mod nmt;
pub mod od;
pub mod pdo;
pub mod sdo;
pub mod sync;

use crate::canopen::sdo::manager::SdoManagerRequest;
use crate::canopen::{emcy::EmergencyMessage, pdo::message::RawPdoMessage, sdo::SdoResponse};
use crate::fifo::Fifo;
use crate::fifo::error::FifoError;
use crate::{
    canopen::{
        nmt::{NmtCommandSpecifier, NmtControlMessage, NmtFrame, NmtMonitorMessage},
        sdo::{SdoDownload, SdoRequest, SdoUpload, frame::SdoFrame},
        sync::SyncMessage,
    },
    cia402::Cia402Identifier,
};
use socketcan::{CanDataFrame, CanFrame, CanSocket, Frame, Socket};
use std::os::fd::{AsRawFd, RawFd};
use tracing::{error, info};

const SDO_EVENT_Q_SIZE: usize = 64;

#[derive(Debug)]
pub enum MessageType {
    NmtControl(NmtControlMessage),
    Sync(SyncMessage), // No node id
    EMCY(EmergencyMessage),
    TSDO(SdoResponse),
    RSDO(SdoRequest),
    PDO(RawPdoMessage),
    NmtMonitor(NmtMonitorMessage),
    Unknown(CanDataFrame), // No node id
}

#[derive(Debug, thiserror::Error)]
pub enum CanOpenError {
    #[error("Unable to write sync")]
    Sync,
    #[error("Unable to send NMT cmd {0:?} to {1:?}")]
    Nmt(NmtCommandSpecifier, Cia402Identifier),
    #[error("Unable to send Sdo Frame {0:?} to {1:?}")]
    Sdo(SdoFrame, Cia402Identifier),
}

/// Thin socketcan wrapper for CANOpen primitives
pub struct CanOpen {
    can: socketcan::CanSocket,
    sync_frame: CanFrame,
    pending_sdo: Fifo<SdoManagerRequest, SDO_EVENT_Q_SIZE>,
}

impl CanOpen {
    pub fn new(can: CanSocket) -> Self {
        const SYNC_ID: u32 = 0x080;
        let sync_frame: CanFrame =
            CanFrame::from_raw_id(SYNC_ID, &[]).expect("failed to construct SYNC frame");

        Self {
            can,
            sync_frame,
            pending_sdo: Fifo::new(),
        }
    }

    pub fn read_raw_frame(&self) -> socketcan::IoResult<CanFrame> {
        let frame = self.can.read_raw_frame()?;
        Ok(frame.into())
    }

    pub fn raw_fd(&self) -> RawFd {
        self.can.as_raw_fd()
    }

    pub fn send_sync(&self) -> Result<(), CanOpenError> {
        self.can
            .write_frame(&self.sync_frame)
            .map_err(|_| CanOpenError::Sync)
    }

    pub fn send_nmt(
        &self,
        cmd: nmt::NmtCommandSpecifier,
        motor: &Cia402Identifier,
    ) -> Result<(), CanOpenError> {
        let frame = NmtFrame::new_cmd_to_node(cmd, motor);
        self.can
            .write_frame(&frame.inner)
            .map_err(|_| CanOpenError::Nmt(cmd, motor.clone()))?;

        Ok(())
    }

    fn send_sdo_download(&self, sdo: &SdoDownload) -> Result<(), CanOpenError> {
        let node = sdo.node;
        let frame = SdoFrame::new_write(sdo);

        self.can
            .write_frame(&frame.inner)
            .map_err(|_| CanOpenError::Sdo(frame, node.clone()))?;

        Ok(())
    }

    fn send_sdo_upload(&self, sdo: &SdoUpload) -> Result<(), CanOpenError> {
        let node = sdo.node;
        let frame = SdoFrame::new_read(sdo);

        self.can
            .write_frame(&frame.inner)
            .map_err(|_| CanOpenError::Sdo(frame, node.clone()))?;

        Ok(())
    }

    pub fn send_single_sdo(&mut self) {
        if let Ok(sdo) = self.pending_sdo.pop() {
            match sdo {
                SdoManagerRequest::Upload(sdo_upload) => self.send_sdo_upload(&sdo_upload),
                SdoManagerRequest::Download(sdo_download) => self.send_sdo_download(&sdo_download),
            };
        }
    }

    pub fn queue_sdo(&mut self, request: SdoManagerRequest) {
        info!(system = "CANOpen", "pushing SDO request: {:?}", request);
        if let Err(e) = self.pending_sdo.push(request) {
            error!(
                system = "CANOpen",
                "Unable to push CANOpen SDO Request: {:?}", e
            );
        }
    }
}

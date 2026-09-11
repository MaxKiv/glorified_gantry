pub mod emcy;
pub mod frame;
pub mod nmt;
pub mod od;
pub mod pdo;
pub mod sdo;
pub mod sync;

use crate::{
    canopen::{
        nmt::{NmtCommandSpecifier, NmtControlMessage, NmtFrame, NmtMonitorMessage},
        sdo::{SdoDownload, SdoUpload, frame::SdoFrame},
        sync::SyncMessage,
    },
    cia402::Cia402Identifier,
};
use socketcan::{CanDataFrame, CanFrame, CanSocket, Frame, Socket};

use crate::canopen::{emcy::EmergencyMessage, pdo::message::RawPdoMessage, sdo::SdoResponse};

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
}

impl CanOpen {
    pub fn new(can: CanSocket) -> Self {
        const SYNC_ID: u32 = 0x080;
        let sync_frame: CanFrame =
            CanFrame::from_raw_id(SYNC_ID, &[]).expect("failed to construct SYNC frame");

        Self { can, sync_frame }
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

    pub fn send_sdo_download(&self, sdo: SdoDownload) -> Result<(), CanOpenError> {
        let node = sdo.node;
        let frame = SdoFrame::new_write(sdo);

        self.can
            .write_frame(&frame.inner)
            .map_err(|_| CanOpenError::Sdo(frame, node.clone()))?;

        Ok(())
    }

    pub fn send_sdo_upload(&self, sdo: SdoUpload) -> Result<(), CanOpenError> {
        let node = sdo.node;
        let frame = SdoFrame::new_read(sdo);

        self.can
            .write_frame(&frame.inner)
            .map_err(|_| CanOpenError::Sdo(frame, node.clone()))?;

        Ok(())
    }
}

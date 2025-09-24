use std::sync::Arc;

/// SDO based Cia402Transport impl for oze-canopen
use oze_canopen::sdo_client::SdoClient;
use tokio::sync::Mutex;
use tracing::*;

use crate::{comms::transport::Cia402Transport, error::DriveError, od::ObjectDictionary};

/// One CANopen SDO parameter write (or read).
#[derive(Debug)]
pub enum SdoAction<'a> {
    /// Send data to device
    Download {
        index: u16,
        subindex: u8,
        data: &'a [u8],
    },
    /// Fetch data from device
    Upload { index: u16, subindex: u8 },
}

pub struct Sdo {
    sdo: Arc<Mutex<SdoClient>>,
}

#[async_trait::async_trait]
impl Cia402Transport for Sdo {
    async fn write_controlword(&self, cw: u16) -> Result<(), DriveError> {
        trace!("SDO controlword writing {cw}");
        let bytes = cw.to_be_bytes();

        trace!("SDO controlword writing bytes: {bytes:?}");
        self.sdo
            .lock()
            .await
            .download(
                ObjectDictionary::CONTROL_WORD.index,
                ObjectDictionary::CONTROL_WORD.sub_index,
                &bytes,
            )
            .await
            .map_err(DriveError::CanOpen)?;

        // TODO: validate the controlword?
        // let dat = s.lock().await.upload(0x607A, 0).await?;

        trace!("SDO controlword write success");
        Ok(())
    }

    async fn read_statusword(&self) -> Result<u16, DriveError> {
        trace!("SDO statusword read start");

        let bytes = self
            .sdo
            .lock()
            .await
            .upload(
                ObjectDictionary::STATUS_WORD.index,
                ObjectDictionary::STATUS_WORD.sub_index,
            )
            .await
            .map_err(DriveError::CanOpen)?;

        trace!("SDO statusword read bytes: {:?}", bytes);

        let sw = u16::from_be_bytes(bytes.try_into().map_err(DriveError::Conversion)?);

        trace!("SDO statusword conversion: {sw}");
        Ok(sw)
    }

    async fn write_operation_mode(&self, mode: u8) -> Result<(), DriveError> {}

    async fn read_operation_mode_display(&self) -> Result<u8, DriveError> {}
}

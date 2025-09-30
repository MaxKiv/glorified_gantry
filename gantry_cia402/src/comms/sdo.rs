use std::sync::Arc;

use oze_canopen::sdo_client::SdoClient;
use tokio::sync::Mutex;

use crate::error::DriveError;

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

impl<'a> SdoAction<'a> {
    pub async fn run_on_sdo_client(&self, sdo: Arc<Mutex<SdoClient>>) -> Result<(), DriveError> {
        match self {
            SdoAction::Download {
                index,
                subindex,
                data,
            } => {
                sdo.lock()
                    .await
                    .download(*index, *subindex, data)
                    .await
                    .map_err(DriveError::CanOpen)?;
            }
            SdoAction::Upload { index, subindex } => {
                sdo.lock()
                    .await
                    .upload(*index, *subindex)
                    .await
                    .map_err(DriveError::CanOpen)?;
            }
        }

        Ok(())
    }
}

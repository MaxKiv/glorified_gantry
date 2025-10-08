use std::{sync::Arc, time::Duration};

use oze_canopen::sdo_client::SdoClient;
use tokio::sync::Mutex;

use crate::{error::DriveError, od::entry::ODEntry};

pub const SDO_PROCESS_DURATION: Duration = Duration::from_millis(0); // Typical SDO round trip at 1mbit/s ~= 4ms, + engineering factor :)

/// One CANopen SDO parameter write (or read).
#[derive(Debug)]
pub enum SdoAction<'a> {
    /// Send data to device
    Download {
        entry: &'static ODEntry,
        data: &'a [u8],
    },
    /// Fetch data from device
    Upload { entry: &'static ODEntry },
}

#[derive(Debug)]
pub enum SdoResult {
    Error(u32),
    None,
    Data(Vec<u8>),
}

#[derive(Debug)]
pub struct SdoTransation<'a> {
    request: &'a SdoAction<'a>,
    result: SdoResult,
}

impl<'a> SdoAction<'a> {
    pub async fn run_on_sdo_client(
        &self,
        sdo: Arc<Mutex<SdoClient>>,
    ) -> Result<SdoTransation<'_>, DriveError> {
        let mut sdo = sdo.lock().await;

        let result = match self {
            SdoAction::Download { entry, data } => {
                sdo.download(entry.index, entry.sub_index, data)
                    .await
                    .map_err(DriveError::CanOpen)?;
                SdoResult::None
            }
            SdoAction::Upload { entry } => {
                let data = sdo
                    .upload(entry.index, entry.sub_index)
                    .await
                    .map_err(DriveError::CanOpen)?;
                SdoResult::Data(data)
            }
        };

        Ok(SdoTransation {
            request: self,
            result,
        })
    }
}

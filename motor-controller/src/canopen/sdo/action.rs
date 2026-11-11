use crate::canopen::od::entry::ODEntry;

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

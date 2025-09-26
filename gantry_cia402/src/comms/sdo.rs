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

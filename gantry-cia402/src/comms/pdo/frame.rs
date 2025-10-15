use tracing::error;

#[derive(Debug)]
/// A single PDO data frame
pub struct PdoFrame {
    /// frame data
    /// Note: Are sent across the wire as is, so make sure to little-endian encode multi-byte values
    pub data: [u8; 8],
    pub dlc: usize,
}

impl PdoFrame {
    pub fn with_dlc(dlc: usize) -> Self {
        Self { data: [0; 8], dlc }
    }

    pub fn set(&mut self, offset: usize, data: &[u8]) {
        if offset + data.len() > self.data.len() {
            error!(
                "Attempting to set RPDO frame data ({:?}) to {data:?} at offset {offset} - but this will exceed RPDO frame length!",
                self
            );
            return;
        }

        self.data[offset..offset + data.len()].copy_from_slice(data);
    }
}

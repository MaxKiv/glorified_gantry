use tracing::error;

use crate::comms::pdo::mapping::{PdoMapping, PdoSet};

#[derive(Debug)]
/// A Collection of PDO data frame that serves as output buffer for [`Pdo`]
pub struct PdoOutputBuffer {
    pub rpdo_frames: [PdoFrame; 4],
}

impl PdoOutputBuffer {
    pub fn from_pdo_set(pdo_set: &PdoSet) -> Self {
        let rpdo_frames: [PdoFrame; 4] = [
            PdoFrame::from_mapping(&pdo_set.rpdos[1]),
            PdoFrame::from_mapping(&pdo_set.rpdos[2]),
            PdoFrame::from_mapping(&pdo_set.rpdos[3]),
            PdoFrame::from_mapping(&pdo_set.rpdos[4]),
        ];

        Self { rpdo_frames }
    }
}

#[derive(Debug)]
/// A single PDO data frame
pub struct PdoFrame {
    /// frame data
    /// NOTE: Are sent across the wire as is, so make sure to little-endian encode multi-byte values
    pub data: [u8; 8],
    /// Data length code - when sending a PdoFrame data[..dlc] is send on the wire
    pub dlc: usize,
    mapping: &'static PdoMapping,
}

impl PdoFrame {
    pub fn from_mapping(mapping: &'static PdoMapping) -> Self {
        Self {
            data: [0; 8],
            dlc: mapping.get_dlc(),
            mapping,
        }
    }

    pub fn get_dlc(&self) -> usize {
        self.dlc
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

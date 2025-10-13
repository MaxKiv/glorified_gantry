pub struct PdoFrame {
    pub data: [u8; 8],
    pub dlc: usize,
}

impl PdoFrame {
    pub fn with_dlc(dlc: usize) -> Self {
        Self { data: [0; 8], dlc }
    }

    pub fn set(&mut self, offset: usize, data: &[u8]) {
        self.data[offset..offset + data.len()].copy_from_slice(data);
    }
}

pub struct PdoFrame {
    pub data: [u8; 8],
}

impl PdoFrame {
    pub fn zero() -> Self {
        Self { data: [0; 8] }
    }

    pub fn set(&mut self, offset: usize, data: &[u8]) {
        self.data[offset..offset + data.len()].copy_from_slice(data);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ODValue {
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    VisibleString([u8; 8]),
    OctetString([u8; 8]),
    Array(usize), // Indicates the presence of sub-indices
    Other,
}

impl ODValue {
    pub fn len_bytes(&self) -> usize {
        match self {
            ODValue::Bool(_) | ODValue::I8(_) | ODValue::U8(_) => 1,
            ODValue::I16(_) | ODValue::U16(_) => 2,
            ODValue::I32(_) | ODValue::U32(_) => 4,
            ODValue::I64(_) | ODValue::U64(_) => 8,
            ODValue::VisibleString(_) | ODValue::OctetString(_) => 8,
            ODValue::Array(n) => *n,
            ODValue::Other => 8,
        }
    }

    /// Maps this ODValue to a byte array
    pub fn to_le_bytes(&self, data: &mut [u8; 8]) -> usize {
        match self {
            ODValue::Bool(b) => {
                data[0] = *b as u8;
                1
            }
            ODValue::I8(v) => {
                data[0] = *v as u8;
                1
            }
            ODValue::U8(v) => {
                data[0] = *v;
                1
            }
            ODValue::I16(v) => {
                data[..2].copy_from_slice(&(*v as u16).to_le_bytes());
                2
            }
            ODValue::U16(v) => {
                data[..2].copy_from_slice(&v.to_le_bytes());
                2
            }
            ODValue::I32(v) => {
                data[..4].copy_from_slice(&(*v as u32).to_le_bytes());
                4
            }
            ODValue::U32(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
                4
            }
            ODValue::I64(v) => {
                *data = (*v as u64).to_le_bytes();
                8
            }
            ODValue::U64(v) => {
                *data = v.to_le_bytes();
                8
            }
            ODValue::VisibleString(s) => {
                let n = s.len().min(8);
                data[..n].copy_from_slice(&s[..n]);
                n
            }
            ODValue::OctetString(bytes) => {
                let n = bytes.len().min(8);
                data[..n].copy_from_slice(&bytes[..n]);
                n
            }
            ODValue::Array(a) => {
                data[..4].copy_from_slice(&(*a as u32).to_le_bytes());
                *a
            }
            ODValue::Other => 0,
        }
    }
}

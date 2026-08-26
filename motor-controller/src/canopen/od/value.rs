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

use crate::od::mappable::MappableType;

use super::{access::AccessType, value::ODValue};

#[derive(Debug, PartialEq)]
pub struct ODEntry {
    // Index of the OD entry
    pub index: u16,
    // Subindex of the OD entry
    pub sub_index: u8,
    // Indicates both the type and default value
    pub default: ODValue,
    pub access: AccessType,
    pub pdo_mappable: MappableType,
}

impl ODEntry {
    pub const fn new(
        index: u16,
        sub_index: u8,
        access: AccessType,
        pdo_mappable: MappableType,
        default: ODValue,
    ) -> Self {
        Self {
            index,
            sub_index,
            access,
            pdo_mappable,
            default,
        }
    }

    pub fn get_num_bytes(&self) -> usize {
        match &self.default {
            ODValue::Bool(_) => 1,
            ODValue::I8(_) => 1,
            ODValue::U8(_) => 1,
            ODValue::I16(_) => 2,
            ODValue::U16(_) => 2,
            ODValue::I32(_) => 4,
            ODValue::U32(_) => 4,
            ODValue::I64(_) => 8,
            ODValue::U64(_) => 8,
            ODValue::F32(_) => 4,
            ODValue::F64(_) => 8,
            ODValue::VisibleString(s) => s.len(),
            ODValue::OctetString(items) => items.len(),
            ODValue::Array(bytes) => *bytes,
        }
    }
}

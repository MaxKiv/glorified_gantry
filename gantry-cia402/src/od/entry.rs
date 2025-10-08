use tracing::*;

use crate::od::{OD_LOOKUP, ODIdx, mappable::MappableType};

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

    // Attempt to parse received data payload into SDO
    pub fn from_sdo_download(data: &[u8; 8], dlc: usize) -> Option<Self> {
        // Extract index/subindex
        let index = u16::from_le_bytes([data[1], data[2]]);
        let sub_index = data[3];
        let idx = ODIdx { index, sub_index };

        // Lookup OD entry
        let entry = OD_LOOKUP.get(&idx)?;
        let expected = &entry.default;

        // Extract payload (after command specifier + index/subindex)
        let payload = &data[4..dlc.min(8)];

        // Try to interpret the bytes into the right ODValue
        let parsed_value = match expected {
            ODValue::Bool(_) => ODValue::Bool(payload[0] != 0),
            ODValue::I8(_) => ODValue::I8(payload[0] as i8),
            ODValue::U8(_) => ODValue::U8(payload[0]),
            ODValue::I16(_) => ODValue::I16(i16::from_le_bytes(payload[..2].try_into().ok()?)),
            ODValue::U16(_) => ODValue::U16(u16::from_le_bytes(payload[..2].try_into().ok()?)),
            ODValue::I32(_) => ODValue::I32(i32::from_le_bytes(payload[..4].try_into().ok()?)),
            ODValue::U32(_) => ODValue::U32(u32::from_le_bytes(payload[..4].try_into().ok()?)),
            ODValue::I64(_) => ODValue::I64(i64::from_le_bytes(
                payload[..8.min(payload.len())].try_into().ok()?,
            )),
            ODValue::U64(_) => ODValue::U64(u64::from_le_bytes(
                payload[..8.min(payload.len())].try_into().ok()?,
            )),
            ODValue::F32(_) => ODValue::F32(f32::from_le_bytes(payload[..4].try_into().ok()?)),
            ODValue::F64(_) => ODValue::F64(f64::from_le_bytes(
                payload[..8.min(payload.len())].try_into().ok()?,
            )),
            _ => {
                error!("Unable to parse {:?} into SDO", data);
                return None;
            }
        };

        Some(Self {
            index,
            sub_index,
            default: parsed_value,
            access: entry.access,
            pdo_mappable: entry.pdo_mappable.clone(),
        })
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

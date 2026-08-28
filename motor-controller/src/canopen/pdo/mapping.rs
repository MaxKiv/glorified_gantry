use tracing::info;

use crate::{
    canopen::{
        od::{
            entry::{ODEntry, PdoSemantic},
            value::ODValue,
        },
        pdo::{PdoType, TransmissionType, message::RawPdoMessage},
    },
    oms::OperationMode,
    rt::MotorFeedback,
    sw::StatusWord,
};

#[derive(Debug, PartialEq, Eq)]
/// Represents a single T/RPDO mapping
pub struct PdoMapping {
    // PDO type and number
    pub pdo: PdoType,
    // Values to map
    pub sources: &'static [PdoMappingSource],
    // When to transmit this PDO
    pub transmission_type: TransmissionType,
}

#[derive(Debug, PartialEq, Eq)]
/// Values to map onto T/RPDO
pub struct PdoMappingSource {
    // The entry to map
    pub entry: &'static ODEntry,
    // Start of bitrange of PDO mapping
    pub start: u8,
    // Length of bitrange of PDO mapping
    pub len: u8,
}

impl PdoMapping {
    // Encodes the given values into the given buffer, ready to be sent over the wire
    // NOTE: this opportunistically encodes values, it is the users responsibility to
    // provide &[`PdoValue`] with the same order and semantic meaning as defined in this mapping
    pub fn encode(&self, values: &[Option<PdoValue>; 4], data: &mut u64) {
        for (i, src) in self.sources.iter().enumerate() {
            let val = values[i].as_ref().expect(
                "provide &[`PdoValue`] with the same order and semantic meaning as defined in this mapping",
            );

            let mask = (src.start as u64) << src.len;
            *data &= val.as_raw() & mask;
        }
    }

    pub fn decode(&self, data: u64) -> [Option<PdoValue>; 4] {
        let mut out = [const { None }; 4];

        for (i, src) in self.sources.iter().enumerate() {
            let mask = (src.start as u64) << src.len;
            let raw = data & mask >> src.len;
            let val = PdoValue::from_raw_semantic(&data, src.entry.semantic);
            out[i] = Some(val);
        }

        out
    }
}

impl PdoMappingSource {
    pub const fn from_od_entry(od_entry: &'static ODEntry, start: u8) -> Self {
        PdoMappingSource {
            entry: &od_entry,
            start,
            len: od_entry.get_num_bits(),
        }
    }
}

/// Operation Mode specific pdo mapping for a single node
#[derive(Default)]
pub struct OMSNodePdoConfig {
    pub tpdo: [Option<PdoMapping>; 4],
    pub rpdo: [Option<PdoMapping>; 4],
}

impl OMSNodePdoConfig {
    pub fn parse_rpdo(
        &self,
        rpdo: RawPdoMessage,
        feedback: &mut MotorFeedback,
    ) -> anyhow::Result<()> {
        match rpdo.pdo_type {
            PdoType::TPDO => anyhow::bail!("parse_rpdo called on TPDO"),
            PdoType::RPDO => {
                let Some(mapping) = &self.rpdo[rpdo.num] else {
                    anyhow::bail!("No mapping for RPDO: {:?}", rpdo);
                };
                let data = u64::from_le_bytes(rpdo.data);
                let decoded = mapping.decode(data);

                for (i, src) in mapping.sources.iter().enumerate() {
                    // Precondition: RPDO messages must contain PdoValues in exactly the same order
                    // as configured in PdoMappingSource
                    let Some(pdo_val) = &decoded[i] else {
                        anyhow::bail!(
                            "parse_rpdo Precondition violated: current rpdo map expects
                            {:?} but this is not present in decoded rpdo: {:?}",
                            src.entry.semantic,
                            decoded
                        );
                    };
                    if src.entry.semantic != pdo_val.semantic {
                        anyhow::bail!(
                            "parse_rpdo Precondition violated: current rpdo map expects
                            {:?} but this is not present in decoded rpdo: {:?}",
                            src.entry.semantic,
                            decoded
                        );
                    }

                    // Match on the semantic meaning of this pdo value & Update MotorFeedback
                    match pdo_val.semantic {
                        PdoSemantic::Statusword => {
                            let sw = StatusWord::from_bits_truncate(pdo_val.as_raw() as u16);
                            info!("parsed statusword: {:?}", sw);
                            feedback.sw = sw
                        }
                        PdoSemantic::ActualOperationMode => {
                            let raw = pdo_val.as_raw() as i8;
                            let opmode = raw.try_into().map_err(|_| {
                                anyhow::anyhow!(
                                    "Failed converting opmode pdo val {} into OperationMode",
                                    raw
                                )
                            })?;
                            info!("parsed opmode: {:?}", opmode);
                            feedback.opmode = opmode;
                        }
                        PdoSemantic::ActualPosition => {
                            let pos = pdo_val.as_raw() as i32;
                            info!("parsed actual position: {:?}", pos);
                            feedback.pos = pos
                        }

                        PdoSemantic::ActualVelocity => {
                            let vel = pdo_val.as_raw() as i32;
                            info!("parsed actual velocity: {:?}", vel);
                            feedback.vel = vel
                        }

                        PdoSemantic::ActualTorque => {
                            let torque = pdo_val.as_raw() as i16;
                            info!("parsed actual torque: {:?}", torque);
                            feedback.torque = torque;
                        }

                        // Unexpected Pdo Value semantic, bail
                        unknown_semantic => anyhow::bail!(
                            "Failed decoding RPDO, unexpected semantic: {:?}",
                            unknown_semantic
                        ),
                    }
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PdoValue {
    pub semantic: PdoSemantic,
    pub value: ODValue,
}

impl PdoValue {
    /// NOTE: Assumes correctly shitfted BE raw value
    pub const fn from_raw_semantic(raw: &u64, semantic: PdoSemantic) -> Self {
        use PdoSemantic::*;

        let value = match semantic {
            ActualOperationMode | TargetOperationMode => ODValue::I8(*raw as i8),
            Statusword | Controlword => ODValue::U16(*raw as u16),
            ActualTorque | TargetTorque => ODValue::I16(*raw as i16),
            ActualPosition | ActualVelocity | TargetPosition | TargetVelocity => {
                ODValue::I32(*raw as i32)
            }
            DeviceType | ProfileVelocity | ProfileAcceleration | ProfileDecceleration => {
                ODValue::U32(*raw as u32)
            }
            DeviceName => ODValue::VisibleString(raw.to_be_bytes()),
            Other => ODValue::Other,
        };

        PdoValue { semantic, value }
    }

    // Encodes the given values into the raw intermediary u64 representation
    pub fn as_raw(&self) -> u64 {
        let out = 0u64;

        let value = match self.value {
            ODValue::Bool(n) => n as u64,
            ODValue::I8(n) => n as u64,
            ODValue::U8(n) => n as u64,
            ODValue::I16(n) => n as u64,
            ODValue::U16(n) => n as u64,
            ODValue::I32(n) => n as u64,
            ODValue::U32(n) => n as u64,
            ODValue::I64(n) => n as u64,
            ODValue::U64(n) => n as u64,
            ODValue::VisibleString(n) => u64::from_be_bytes(n),
            ODValue::OctetString(n) => u64::from_be_bytes(n),
            ODValue::Array(n) => n as u64,
            ODValue::Other => out,
        };

        value
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        consts::pdo::pp::DEFAULT_PP_PDOCFG, cw::ControlWord, oms::OperationMode,
        utils::setup_tracing_subscriber,
    };

    use super::*;
    use tracing::*;

    #[test]
    fn pdo_mapping_encode_decode() -> anyhow::Result<()> {
        let cfg = DEFAULT_PP_PDOCFG;

        Ok(())
    }

    #[test]
    fn pdo_value_encode_decode() -> anyhow::Result<()> {
        setup_tracing_subscriber();

        let cw = ControlWord::from_bits_truncate(0b1111_0000_1111_0000);

        // Encode cw & om in u64 intermediary representation, add trash data bits
        let opmode = OperationMode::ProfilePosition;
        let opmode_bits: i8 = opmode as i8;
        let mut data_good = cw.bits() as u64 ^ ((opmode as u64) << 16) ^ (0xDEAFBEADu64 << 24);

        info!("CW: {:#x}->{:?}", cw.bits(), cw);
        info!("OM: {:#x}->{:?}", opmode_bits, opmode);
        info!("Data: {:#x}", data_good);

        // Convert cw into PdoValue, check if cw is properly encoded
        let val = PdoValue::from_raw_semantic(&data_good, PdoSemantic::Controlword);
        info!("Decoded CW -> {:?}", val);
        assert!(val.value == ODValue::U16(cw.bits()));

        // Convert back into CW, check if properly decoded
        let cw_decoded = ControlWord::from_bits_truncate(val.as_raw() as u16);
        assert!(cw == cw_decoded);

        // Convert om into PdoValue, check if properly encoded
        data_good >>= 16;
        let val = PdoValue::from_raw_semantic(&data_good, PdoSemantic::TargetOperationMode);
        let ODValue::I8(n) = val.value else {
            panic!(
                "PdoValue::from_raw_semantic(&data_good, PdoSemantic::TargetOperationMode) returned wrong ODValue type"
            );
        };
        let om_pdovalue = OperationMode::try_from(n).unwrap();
        info!("Decoded OM -> {:?} -> {:?}", val, om_pdovalue);
        assert!(opmode == om_pdovalue);

        // Convert back into OM, check if properly decoded
        let om_decoded = OperationMode::try_from(val.as_raw() as i8).unwrap();
        assert!(om_decoded == opmode);

        let data_bad = (cw.bits() as u64) << 1;
        let val = PdoValue::from_raw_semantic(&data_bad, PdoSemantic::Controlword);
        assert!(val.value != ODValue::U16(cw.bits()));
        let cw_decoded = ControlWord::from_bits_truncate(val.as_raw() as u16);
        assert!(cw != cw_decoded);

        Ok(())
    }
}

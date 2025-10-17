use crate::driver::{
    oms::{OMSFlagsSW, OperationMode},
    receiver::{
        StatusWord,
        parse::{ParseError, pdo_message::*},
    },
    update::ControlWord,
};

impl TryFrom<[u8; 8]> for TPDO1Message {
    type Error = ParseError;

    fn try_from(value: [u8; 8]) -> Result<Self, Self::Error> {
        let statusword = StatusWord::from_bits_truncate(u16::from_le_bytes([value[0], value[1]]));

        const OPMODE_BYTE: usize = 2;
        let opmode = value[OPMODE_BYTE] as i8;
        let actual_opmode: OperationMode = opmode.try_into().map_err(|_| {
            ParseError(anyhow::anyhow!(
                "Failed to parse operation mode from TPDO1 data: {value:?}"
            ))
        })?;

        let oms_flags = OMSFlagsSW::from_statusword_and_opmode(statusword, actual_opmode);

        Ok(TPDO1Message {
            statusword,
            actual_opmode,
            oms_flags,
        })
    }
}

impl TryFrom<[u8; 8]> for TPDO2Message {
    type Error = ParseError;

    fn try_from(value: [u8; 8]) -> Result<Self, Self::Error> {
        let actual_pos = i32::from_le_bytes([value[0], value[1], value[2], value[3]]);
        let actual_vel = i32::from_le_bytes([value[4], value[5], value[6], value[7]]);

        Ok(TPDO2Message {
            actual_pos,
            actual_vel,
        })
    }
}

impl TryFrom<[u8; 8]> for TPDO3Message {
    type Error = ParseError;

    fn try_from(value: [u8; 8]) -> Result<Self, Self::Error> {
        let actual_torque = i16::from_le_bytes([value[0], value[1]]);

        Ok(TPDO3Message { actual_torque })
    }
}

impl TryFrom<[u8; 8]> for TPDO4Message {
    type Error = ParseError;

    fn try_from(_: [u8; 8]) -> Result<Self, Self::Error> {
        Ok(TPDO4Message)
    }
}

impl TryFrom<[u8; 8]> for RPDO1Message {
    type Error = ParseError;

    fn try_from(value: [u8; 8]) -> Result<Self, Self::Error> {
        let controlword = ControlWord::from_bits_truncate(u16::from_le_bytes([value[0], value[1]]));

        const OPMODE_BYTE: usize = 2;
        let opmode = value[OPMODE_BYTE] as i8;
        let opmode: OperationMode = opmode.try_into().map_err(|_| {
            ParseError(anyhow::anyhow!(
                "Failed to parse operation mode from RPDO1 data: {value:?}"
            ))
        })?;

        Ok(RPDO1Message {
            controlword,
            opmode,
        })
    }
}

impl TryFrom<[u8; 8]> for RPDO2Message {
    type Error = ParseError;

    fn try_from(value: [u8; 8]) -> Result<Self, Self::Error> {
        let target_pos = i32::from_le_bytes([value[0], value[1], value[2], value[3]]);
        let profile_velocity = u32::from_le_bytes([value[4], value[5], value[6], value[7]]);

        Ok(RPDO2Message {
            target_pos,
            profile_velocity,
        })
    }
}

impl TryFrom<[u8; 8]> for RPDO3Message {
    type Error = ParseError;

    fn try_from(value: [u8; 8]) -> Result<Self, Self::Error> {
        let target_velocity = i32::from_le_bytes([value[0], value[1], value[2], value[3]]);

        Ok(RPDO3Message { target_velocity })
    }
}

impl TryFrom<[u8; 8]> for RPDO4Message {
    type Error = ParseError;

    fn try_from(value: [u8; 8]) -> Result<Self, Self::Error> {
        let target_torque = i16::from_le_bytes([value[0], value[1]]);

        Ok(RPDO4Message { target_torque })
    }
}

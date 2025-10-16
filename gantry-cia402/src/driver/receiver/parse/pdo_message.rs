
use oze_canopen::canopen::NodeId;

use crate::{
    comms::pdo::mapping::PdoType,
    driver::{oms::OperationMode, receiver::StatusWord, update::ControlWord},
};

#[derive(Debug, Clone)]
pub enum PDOMessage {
    TPDO1(TPDO1Message),
    TPDO2(TPDO2Message),
    TPDO3(TPDO3Message),
    TPDO4(TPDO4Message),
    RPDO1(RPDO1Message),
    RPDO2(RPDO2Message),
    RPDO3(RPDO3Message),
    RPDO4(RPDO4Message),
    Raw(RawPDOMessage),
}

#[derive(Debug, Clone)]
pub struct ParsedPDO {
    pub node: NodeId,
    pub num: u8,
    pub kind: PdoType,
    pub message: PDOMessage,
    pub raw_data: [u8; 8],
    pub raw_dlc: usize,
}

pub struct PrettyPdo {
    pub header: String,
    pub raw: String,
    pub parsed: String,
}

impl From<ParsedPDO> for PrettyPdo {
    fn from(value: ParsedPDO) -> Self {
        let header = value.kind.to_string_pretty();
        let raw = format!("{0:x?}", value.raw_data[..value.raw_dlc].to_vec());

        let parsed = match &value.message {
            PDOMessage::TPDO1(m) => format!("{0:?} - {1:?}", m.statusword, m.actual_opmode),
            PDOMessage::TPDO2(m) => format!("pos: {0:?} - vel: {1:?}", m.actual_pos, m.actual_vel),
            PDOMessage::TPDO3(m) => format!("torque {0:?}", m.actual_torque),
            PDOMessage::TPDO4(_) => String::new(),
            PDOMessage::RPDO1(m) => format!("{0:?} - {1:?}", m.controlword, m.opmode),
            PDOMessage::RPDO2(m) => format!(
                "target pos: {0:?} - profile vel: {1:?}",
                m.target_pos, m.profile_velocity
            ),
            PDOMessage::RPDO3(m) => format!("target vel: {0:?}", m.target_velocity),
            PDOMessage::RPDO4(m) => format!("target torque: {0:?}", m.target_torque),
            PDOMessage::Raw(_) => String::new(),
        };

        PrettyPdo {
            header,
            raw,
            parsed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RawPDOMessage {
    pub cob_id: usize,
    pub data: [u8; 8],
    pub dlc: usize,
}

#[derive(Debug, Clone)]
pub struct TPDO1Message {
    pub statusword: StatusWord,
    pub actual_opmode: OperationMode,
}

#[derive(Debug, Clone)]
pub struct TPDO2Message {
    pub actual_pos: i32,
    pub actual_vel: i32,
}

#[derive(Debug, Clone)]
pub struct TPDO3Message {
    pub actual_torque: i16,
}

#[derive(Debug, Clone)]
pub struct TPDO4Message;

#[derive(Debug, Clone)]
pub struct RPDO1Message {
    pub controlword: ControlWord,
    pub opmode: OperationMode,
}

#[derive(Debug, Clone)]
pub struct RPDO2Message {
    pub target_pos: i32,
    pub profile_velocity: u32,
}

#[derive(Debug, Clone)]
pub struct RPDO3Message {
    pub target_velocity: i32,
}

#[derive(Debug, Clone)]
pub struct RPDO4Message {
    pub target_torque: i16,
}

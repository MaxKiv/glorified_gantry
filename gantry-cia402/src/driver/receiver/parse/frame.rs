use oze_canopen::canopen::RxMessage;
use tracing::error;

use crate::{
    comms::pdo::mapping::PdoType,
    driver::{
        nmt::NmtState,
        receiver::parse::{pdo_message::*, *},
        *,
    },
};

impl TryFrom<RxMessage> for Frame {
    type Error = ParseError;

    fn try_from(frame: RxMessage) -> Result<Frame, ParseError> {
        let id = frame.cob_id;
        let timestamp = frame.timestamp;

        let (node_id, message) = match id {
            // 0x000 → NMT Command
            0x000 => {
                let requested_state = match frame.data[0] {
                    0x01 => NmtState::Operational,
                    0x02 => NmtState::Stopped,
                    0x80 => NmtState::PreOperational,
                    _ => NmtState::PreOperational,
                };
                let node_id = Some(frame.data[1]);

                (
                    node_id,
                    MessageType::NmtControl(NmtControlMessage { requested_state }),
                )
            }

            // 0x080 → SYNC
            0x080 => (None, MessageType::Sync(SyncMessage)),

            // 0x081–0x0FF → EMCY (Emergency)
            0x081..=0x0FF => {
                let node_id = Some((id - 0x080) as u8);
                let error_code = u16::from_le_bytes([frame.data[0], frame.data[1]]);
                error!(
                    "Error frame received: {frame:?} - error code: {:#0x} - see datasheet page 108",
                    error_code
                );

                let error = match error_code {
                    0x0 => EMCY::NoFurtherPendingErrors,
                    0x3100 => EMCY::Undervoltage,
                    0x8210 => EMCY::PdoLengthError,
                    0x8220 => EMCY::PdoLengthExceeded,
                    0x5440 => EMCY::InterlockError,
                    0x6010 => EMCY::SoftwareReset,
                    0x6100 => EMCY::InternalSoftwareError,
                    0x6320 => EMCY::RatedCurrentNotSet,
                    0x7113 => EMCY::BallastResistorOverload,
                    0x7121 => EMCY::MotorBlocked,
                    0x7200 => EMCY::InternalCorrectionFactorMissing,
                    0x7305 => EMCY::Sensor1Fault,
                    0x7306 => EMCY::Sensor2Fault,
                    0x7307 => EMCY::SensorNFault,
                    0x7600 => EMCY::NonvolatileMemoryFull,
                    0x8100 => EMCY::FieldbusError,
                    0x8130 => EMCY::HeartbeatError,
                    0x8200 => EMCY::SlaveTimeout,
                    0x8240 => EMCY::UnexpectedSyncLength,
                    0x8400 => EMCY::SpeedMonitoringError,
                    0x8611 => EMCY::FollowingErrorTooLarge,
                    0x8612 => EMCY::LimitSwitchExceeded,
                    _ => EMCY::Unknown,
                };

                (node_id, MessageType::EMCY(EmergencyMessage { error }))
            }

            // TPDO1..4 (0x180 + n*0x200)
            0x180..=0x57F => {
                let (kind, base) = match id {
                    0x180..=0x1FF => (PdoType::TPDO(1), 0x180),
                    0x200..=0x21F => (PdoType::RPDO(1), 0x200),
                    0x280..=0x2FF => (PdoType::TPDO(2), 0x280),
                    0x300..=0x31F => (PdoType::RPDO(2), 0x300),
                    0x380..=0x3FF => (PdoType::TPDO(3), 0x380),
                    0x400..=0x41F => (PdoType::RPDO(3), 0x400),
                    0x480..=0x4FF => (PdoType::TPDO(4), 0x480),
                    0x500..=0x51F => (PdoType::RPDO(4), 0x500),
                    _ => unreachable!(),
                };
                // let node_id = Some((id - base) as u8);
                let node = (id - base) as u8;

                let (num, message) = match kind {
                    PdoType::RPDO(1) => (1, PDOMessage::RPDO1(frame.data.try_into()?)),
                    PdoType::RPDO(2) => (2, PDOMessage::RPDO2(frame.data.try_into()?)),
                    PdoType::RPDO(3) => (3, PDOMessage::RPDO3(frame.data.try_into()?)),
                    PdoType::RPDO(4) => (4, PDOMessage::RPDO4(frame.data.try_into()?)),
                    PdoType::TPDO(1) => (1, PDOMessage::TPDO1(frame.data.try_into()?)),
                    PdoType::TPDO(2) => (2, PDOMessage::TPDO2(frame.data.try_into()?)),
                    PdoType::TPDO(3) => (3, PDOMessage::TPDO3(frame.data.try_into()?)),
                    PdoType::TPDO(4) => (4, PDOMessage::TPDO4(frame.data.try_into()?)),
                    _ => (
                        255,
                        PDOMessage::Raw(RawPDOMessage {
                            cob_id: id as usize,
                            data: frame.data,
                            dlc: frame.dlc,
                        }),
                    ),
                };

                let parsed = ParsedPDO {
                    node,
                    num,
                    kind,
                    message,
                    raw_data: frame.data,
                    raw_dlc: frame.dlc,
                };

                (Some(node), MessageType::PDO(parsed))
            }

            // 0x580–0x5FF → TSDO (Server→Client)
            0x580..=0x5FF => {
                // let value = ODEntry::from_sdo_download(&frame.data, frame.dlc);
                let node_id = Some((frame.cob_id - 0x580) as u8);
                let response = SdoResponse::from_frame(&frame).map_err(ParseError)?;

                (node_id, MessageType::TSDO(response))
            }

            // 0x600–0x67F → RSDO (Client→Server)
            0x600..=0x67F => {
                let node_id = Some((id - 0x600) as u8);
                let value = ODEntry::from_sdo_download(&frame.data, frame.dlc);

                (
                    node_id,
                    MessageType::RSDO(SdoRequest {
                        data: frame.data,
                        dlc: frame.dlc,
                        value,
                    }),
                )
            }

            // 0x700–0x77F → Heartbeat / Node Monitoring
            0x700..=0x77F => {
                let node_id = Some((id - 0x700) as u8);
                let current_state = match frame.data[0] {
                    0x00 => NmtState::Bootup,
                    0x04 => NmtState::Stopped,
                    0x05 => NmtState::Operational,
                    0x7F => NmtState::PreOperational,
                    _ => NmtState::PreOperational,
                };
                (
                    node_id,
                    MessageType::NmtMonitor(NmtMonitorMessage { current_state }),
                )
            }

            _ => (None, MessageType::Unknown(frame)),
        };

        Ok(Frame {
            timestamp,
            node_id,
            message,
        })
    }
}

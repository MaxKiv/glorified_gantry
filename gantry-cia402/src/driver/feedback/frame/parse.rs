use oze_canopen::canopen::RxMessage;

use crate::driver::{feedback::frame::*, nmt::NmtState};

impl TryFrom<RxMessage> for Frame {
    type Error = ();

    fn try_from(value: RxMessage) -> Result<Self, Self::Error> {
        let id = value.cob_id;
        let timestamp = value.timestamp;

        Ok(match id {
            // 0x000 → NMT Command
            0x000 => {
                let requested_state = match value.data[0] {
                    0x01 => NmtState::Operational,
                    0x02 => NmtState::Stopped,
                    0x80 => NmtState::PreOperational,
                    _ => NmtState::PreOperational,
                };
                let to = value.data[1];
                Frame::NmtControl(NmtControlMessage {
                    timestamp,
                    requested_state,
                    to,
                })
            }

            // 0x080 → SYNC
            0x080 => Frame::Sync(SyncMessage { timestamp }),

            // 0x081–0x0FF → EMCY (Emergency)
            0x081..=0x0FF => {
                let node_id = (id - 0x080) as u8;
                let error_code = u16::from_le_bytes([value.data[0], value.data[1]]);
                let error = match error_code {
                    0x3100 => EMCY::Undervoltage,
                    _ => EMCY::Unknown,
                };
                Frame::EMCY(EmergencyMessage {
                    timestamp,
                    from: node_id,
                    error,
                })
            }

            // TPDO1..4 (0x180 + n*0x200)
            0x180..=0x57F => {
                let (pdo, base) = match id {
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
                let from = (id - base) as u8;

                match pdo {
                    PdoType::TPDO(num) => Frame::TPDO(TPDOMessage {
                        timestamp,
                        from,
                        num: num.into(),
                        data: value.data,
                        dlc: value.dlc,
                    }),
                    PdoType::RPDO(num) => Frame::RPDO(RPDOMessage {
                        timestamp,
                        from,
                        num: num.into(),
                        data: value.data,
                        dlc: value.dlc,
                    }),
                }
            }

            // 0x580–0x5FF → TSDO (Server→Client)
            0x580..=0x5FF => {
                let from = (id - 0x580) as u8;
                Frame::TSDO(SdoMessage {
                    timestamp,
                    from,
                    data: value.data,
                    dlc: value.dlc,
                })
            }

            // 0x600–0x67F → RSDO (Client→Server)
            0x600..=0x67F => {
                let from = (id - 0x600) as u8;
                Frame::RSDO(SdoMessage {
                    timestamp,
                    from,
                    data: value.data,
                    dlc: value.dlc,
                })
            }

            // 0x700–0x77F → Heartbeat / Node Monitoring
            0x700..=0x77F => {
                let from = (id - 0x700) as u8;
                let current_state = match value.data[0] {
                    0x00 => NmtState::Bootup,
                    0x04 => NmtState::Stopped,
                    0x05 => NmtState::Operational,
                    0x7F => NmtState::PreOperational,
                    _ => NmtState::PreOperational,
                };
                Frame::NmtMonitor(NmtMonitorMessage {
                    timestamp,
                    from,
                    current_state,
                })
            }

            _ => Frame::Unknown(value),
        })
    }
}

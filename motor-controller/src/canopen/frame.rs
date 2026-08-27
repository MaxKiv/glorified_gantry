use socketcan::{CanDataFrame, CanFrame, EmbeddedFrame};
use tracing::error;

use crate::canopen::{
    EMCY, EmergencyMessage, MessageType, NmtControlMessage, NmtMonitorMessage, NmtState,
    SyncMessage,
    od::entry::ODEntry,
    pdo::message::RawPdoMessage,
    sdo::{SdoRequest, SdoResponse},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CobId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// CANOpen NodeId
pub struct NodeId(pub u8);

#[derive(Debug)]
pub struct CanOpenFrame {
    pub cob_id: CobId,
    pub msg: MessageType,
}

#[derive(Debug, thiserror::Error)]
pub enum CanOpenParseError {
    #[error("Unable to parse socketcan Remote Frame")]
    RemoteFrame,
    #[error("Unable to parse socketcan Error Frame")]
    ErrorFrame,
    #[error("Tried to parse an extended CAN frame")]
    ExtendedCANFrame,
    #[error("Unable to parse frame with large dlc: {0}")]
    ExceededMaxDLC(usize),
    #[error(
        "Attempt to parse message {0:?}, its COB-ID is within T/RPDO range but doesnt map to #1-4?"
    )]
    PdoNumRange(CanDataFrame),
    #[error("Unexpected DLC: {0}")]
    WrongDLC(usize),
    #[error("Unable to parse SDO: {0:?}")]
    SdoInvalidData(CanDataFrame),
}

impl CanOpenFrame {
    /// Parses a [`socketcan::CanFrame`] into a [`CanOpenFrame`]
    pub fn from_canframe(frame: CanFrame) -> Result<Self, CanOpenParseError> {
        pub const MAX_DLC: usize = 8;

        match frame {
            CanFrame::Remote(_) => Err(CanOpenParseError::RemoteFrame),
            CanFrame::Error(_) => Err(CanOpenParseError::ErrorFrame),
            CanFrame::Data(frame) => {
                // Get frame id & make sure its standard 11 bit length
                let socketcan::Id::Standard(id) = frame.id() else {
                    return Err(CanOpenParseError::ExtendedCANFrame);
                };
                let cob_id = CobId(id.as_raw());

                let frame_data = frame.data();
                let frame_dlc = frame.dlc();
                if frame_dlc > MAX_DLC {
                    return Err(CanOpenParseError::ExceededMaxDLC(frame_dlc));
                }

                let msg = match cob_id.0 {
                    // 0x000 -> NMT Command
                    0x000 => {
                        let requested_state = NmtState::from_nmt_command_frame(&frame_data);
                        let node_id = NodeId(frame_data[1]);

                        MessageType::NmtControl(NmtControlMessage {
                            requested_state,
                            node_id,
                        })
                    }

                    // 0x080 -> SYNC
                    0x080 => MessageType::Sync(SyncMessage),

                    // 0x081–0x0FF -> EMCY (Emergency)
                    0x081..=0x0FF => {
                        let node_id = NodeId((cob_id.0 - 0x080) as u8);
                        let error_code = u16::from_le_bytes([frame_data[0], frame_data[1]]);
                        error!(
                            "Error / EMCY frame received: {frame:?} - error code: {:#0x} - see datasheet page 108",
                            error_code
                        );

                        let error = EMCY::from_error_code(error_code);

                        MessageType::EMCY(EmergencyMessage { error, node_id })
                    }

                    // T/RPDO1..4 (0x180 + n*0x200)
                    0x180..=0x57F => {
                        let msg = RawPdoMessage::try_from_can_frame(cob_id, frame)?;
                        MessageType::PDO(msg)
                    }

                    // 0x580–0x5FF -> TSDO
                    0x580..=0x5FF => {
                        let response = SdoResponse::try_from_frame(cob_id, &frame)?;
                        MessageType::TSDO(response)
                    }

                    // 0x600–0x67F -> RSDO
                    0x600..=0x67F => {
                        let od_entry = ODEntry::from_sdo_download(frame_data, frame_dlc);
                        let mut data = [0u8; 8];
                        data.copy_from_slice(frame_data);

                        MessageType::RSDO(SdoRequest {
                            data,
                            dlc: frame_dlc,
                            value: od_entry,
                        })
                    }

                    // 0x700–0x77F -> Heartbeat / Node Monitoring
                    0x700..=0x77F => {
                        let node_id = NodeId((cob_id.0 - 0x700) as u8);

                        let current_state = NmtState::from_node_monitoring_frame(&frame_data);

                        MessageType::NmtMonitor(NmtMonitorMessage {
                            current_state,
                            node_id,
                        })
                    }

                    _ => MessageType::Unknown(frame),
                };

                Ok(Self { cob_id, msg })
            }
        }
    }
}

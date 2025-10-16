use tracing::*;

use crate::{
    comms::pdo::mapping::PdoType,
    driver::receiver::parse::{MessageType, pdo_message::PrettyPdo, *},
};

// 1) change your Frame::log to emit structured fields (no ANSI)
impl Frame {
    pub fn log(&self) {
        match &self.message {
            MessageType::EMCY(msg) => {
                // note: record node as u64 so Visit::record_u64 catches it
                error!(
                    target: "canopen",
                    frame = "EMCY",
                    node = self.node_id.unwrap_or(0) as u64,
                    error = ?msg.error,
                    message = %format!("{:?}", msg.error)
                );
            }
            MessageType::Sync(_) => info!(target: "canopen", frame = "SYNC"),
            MessageType::NmtMonitor(msg) => {
                info!(
                    target: "canopen",
                    frame = "NmtMonitor",
                    node = self.node_id.unwrap_or(0) as u64,
                    data = %format!("{:?}", msg.current_state)
                );
            }
            MessageType::NmtControl(msg) => {
                info!(target: "canopen", frame = "NmtControl", node = self.node_id.unwrap_or(0) as u64, data =
                    %format!("{:?}", msg.requested_state));
            }
            MessageType::TSDO(msg) => {
                info!(
                    target: "canopen",
                    frame = "TSDO",
                    node = self.node_id.unwrap_or(0) as u64,
                    parsed = msg.fmt_pretty()
                );
            }
            MessageType::RSDO(msg) => {
                info!(
                    target: "canopen",
                    frame = "RSDO",
                    node = self.node_id.unwrap_or(0) as u64,
                    data = %hex_dump(&msg.data[..msg.dlc]),
                    parsed = ?msg.value,
                );
            }
            MessageType::PDO(msg) => {
                use PdoType::*;

                let pretty: PrettyPdo = msg.clone().into();

                let frame = match msg.kind {
                    RPDO(_) => "RPDO",
                    TPDO(_) => "TPDO",
                };

                info!(
                    target: "canopen",
                    frame = frame,
                    node = self.node_id.unwrap_or(0) as u64,
                    data = pretty.raw,
                    parsed = pretty.parsed,
                    header = pretty.header,
                );
            }
            MessageType::Unknown(rx_message) => {
                error!(
                        target: "canopen",
                        frame = "Unknown",
                        node = rx_message.cob_id as u64,
                        data = %hex_dump(&rx_message.data[..rx_message.dlc])

                );
            }
        }
    }
}

pub fn hex_dump(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:#2X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

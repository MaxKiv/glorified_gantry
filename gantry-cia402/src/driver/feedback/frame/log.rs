use tracing::*;

use crate::driver::feedback::frame::Frame;

// 1) change your Frame::log to emit structured fields (no ANSI)
impl Frame {
    pub fn log(&self) {
        match self {
            Frame::EMCY(msg) => {
                // note: record node as u64 so Visit::record_u64 catches it
                error!(
                    target: "canopen",
                    frame = "EMCY",
                    node = msg.from as u64,
                    error = ?msg.error,
                    message = %format!("{:?}", msg.error)
                );
            }
            Frame::TPDO(msg) => {
                info!(
                    target: "canopen",
                    frame = "TPDO",
                    node = msg.from as u64,
                    num = msg.num as u64,
                    data = %hex_dump(&msg.data[..msg.dlc])
                );
            }
            Frame::Sync(_) => info!(target: "canopen", frame = "SYNC"),
            Frame::RPDO(msg) => info!(
                target: "canopen",
                frame = "RPDO",
                node = msg.from as u64,
                num = msg.num as u64,
                data = %hex_dump(&msg.data[..msg.dlc])
            ),
            Frame::NmtMonitor(msg) => {
                info!(
                    target: "canopen",
                    frame = "NmtMonitor",
                    node = msg.from as u64,
                    data = %format!("{:?}", msg.current_state)
                );
            }
            Frame::NmtControl(msg) => {
                info!(target: "canopen", frame = "NmtControl", node = msg.to as u64, data =
                    %format!("{:?}", msg.requested_state));
            }
            Frame::TSDO(msg) => {
                info!(
                    target: "canopen",
                    frame = "TSDO",
                    node = msg.from as u64,
                    data = %hex_dump(&msg.data[..msg.dlc])
                );
            }
            Frame::RSDO(msg) => {
                info!(
                    target: "canopen",
                    frame = "RSDO",
                    node = msg.from as u64,
                    data = %hex_dump(&msg.data[..msg.dlc])
                );
            }
            Frame::Unknown(rx_message) => {
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

fn hex_dump(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:#2X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

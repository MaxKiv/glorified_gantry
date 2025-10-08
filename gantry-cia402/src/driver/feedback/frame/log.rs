use tracing::*;

use crate::driver::feedback::frame::{Frame, *};

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
                let from = match msg {
                    SdoResponse::Error(sdo_error) => sdo_error.from,
                    SdoResponse::DownloadConfirm(sdo_download_confirmed) => {
                        sdo_download_confirmed.from
                    }
                    SdoResponse::UploadConfirm(sdo_upload_result) => sdo_upload_result.from,
                };

                // match msg {
                // SdoResponse::Error(sdo_error) => {
                //     info!(
                //         target: "canopen",
                //         frame = "TSDO",
                //         node = sdo_error.from,
                //         index = sdo_error.index as u64,
                //         sub_index = sdo_error.sub_index,
                //         data = sdo_error.code,
                //         parsed = ?sdo_error,
                //     );
                // }
                // SdoResponse::DownloadConfirm(sdo_download_result) => {
                //     info!(
                //         target: "canopen",
                //         frame = "TSDO",
                //         node = sdo_download_result.from,
                //         index = sdo_download_result.index as u64,
                //         sub_index = sdo_download_result.sub_index,
                //         parsed = ?sdo_download_result,
                //     );
                // }
                // SdoResponse::UploadConfirm(sdo_upload_result) => {
                //     info!(
                //         target: "canopen",
                //         frame = "TSDO",
                //         node = sdo_upload_result.from,
                //         index = sdo_upload_result.index as u64,
                //         sub_index = sdo_upload_result.sub_index,
                //         data = %hex_dump(&sdo_upload_result.data),
                //         parsed = ?sdo_upload_result,
                //     );
                // }
                info!(
                    target: "canopen",
                    frame = "TSDO",
                    node = from,
                    parsed = msg.fmt_pretty()
                );
            }
            Frame::RSDO(msg) => {
                info!(
                    target: "canopen",
                    frame = "RSDO",
                    node = msg.from as u64,
                    data = %hex_dump(&msg.data[..msg.dlc]),
                    parsed = ?msg.value,
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

pub fn hex_dump(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:#2X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

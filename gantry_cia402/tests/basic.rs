mod common;

use std::time::Duration;

use oze_canopen::{
    error::CoError,
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
};
use tracing::*;

const NODE_ID: u8 = 3;

/// Quick test of oze-canopen
/// Attempts some SDO down/uploads to a single node
/// Useful to see if your socketCAN setup is correct (if not: run `just setup-can`)
async fn test_oze_canopen() -> Result<(), CoError> {
    info!("Starting can interface");
    let (interface, mut handles) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

    info!("Starting test, Sending NMT Operational to node id {NODE_ID}");
    // Motor boots into NMT::PreOperational -> Set motor to NMT::Operational
    interface
        .send_nmt(NmtCommand::new(
            NmtCommandSpecifier::StartRemoteNode,
            NODE_ID,
        ))
        .await?;

    // Give the slave device some time to boot, we all have trouble getting out of bed sometimes
    tokio::time::sleep(Duration::from_millis(200)).await;

    info!("Getting sdo client");
    let s = interface.get_sdo_client(3).unwrap();

    info!("Testing upload");
    let dat = s.lock().await.upload(0x1000, 0).await?;
    info!("Test upload - device type: {dat:?}");

    info!("Testing download");
    let dat = s.lock().await.upload(0x607A, 0).await?;
    info!("Test before download - target position: {dat:?} - Setting to 4x 0x0");

    s.lock()
        .await
        .download(0x607A, 0, &[0x00, 0x00, 0x00, 0x00])
        .await?;
    let dat = s.lock().await.upload(0x607A, 0).await?;
    info!("Test before download - target position: {dat:?} - Setting to [0x1, 0x2, 0x3, 0x4]");

    s.lock()
        .await
        .download(0x607A, 0, &[0x1, 0x2, 0x3, 0x4])
        .await?;
    let dat = s.lock().await.upload(0x607A, 0).await?;
    info!("Test after download - target position: {dat:?}");

    // stop tasks
    handles.close_and_join().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic_canopen_test() -> Result<(), CoError> {
        common::setup_tracing();

        test_oze_canopen().await?;

        Ok(())
    }
}

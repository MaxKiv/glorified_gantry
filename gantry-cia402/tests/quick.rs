mod common;

use std::time::Duration;

use gantry_cia402::od::{MAX_CURRENT, STATUS_WORD, TORQUE_SLOPE};
use oze_canopen::proto::nmt::{NmtCommand, NmtCommandSpecifier};
use tracing::*;

use crate::common::TestError;

const NODE_ID: u8 = 3;

/// Quick test of oze-canopen
/// Attempts some SDO down/uploads to a single node
/// Useful to see if your socketCAN setup is correct (if not: run `just setup-can`)
async fn quick_test_logic() -> Result<(), TestError> {
    info!("Starting can interface");
    let (interface, mut handles) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

    info!("Starting test, Sending NMT Operational to node id {NODE_ID}");
    // Motor boots into NMT::PreOperational -> Set motor to NMT::Operational
    interface
        .send_nmt(NmtCommand::new(
            NmtCommandSpecifier::StartRemoteNode,
            NODE_ID,
        ))
        .await
        .map_err(TestError::CANOpenError)?;

    // Give the slave device some time to boot, we all have trouble getting out of bed sometimes
    tokio::time::sleep(Duration::from_millis(200)).await;

    info!("Getting sdo client");
    let s = interface.get_sdo_client(3).unwrap();

    info!("Testing upload");
    let dat = s
        .lock()
        .await
        .upload(0x1000, 0)
        .await
        .map_err(TestError::CANOpenError)?;

    info!("Test upload - device type: {dat:?}");

    let dat = s
        .lock()
        .await
        .upload(MAX_CURRENT.index, MAX_CURRENT.sub_index)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u16::from_le_bytes([dat[0], dat[1]]);
    info!("Device Max current: {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(TORQUE_SLOPE.index, TORQUE_SLOPE.sub_index)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u32::from_le_bytes(dat[..4].try_into().map_err(|_| TestError::Generic)?);
    info!("Torque slope: {}={:#x}", val, val);

    // stop tasks
    handles.close_and_join().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn quick_test() -> Result<(), TestError> {
        gantry_demo::setup_tracing();

        quick_test_logic().await?;

        Ok(())
    }
}

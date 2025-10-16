mod common;

use std::time::Duration;

use gantry_cia402::{driver::receiver::StatusWord, od::STATUS_WORD};
use oze_canopen::{
    error::CoError,
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
};
use tracing::*;

use crate::common::TestError;

const NODE_ID: u8 = 3;

/// Quick test of oze-canopen
/// Attempts some SDO down/uploads to a single node
/// Useful to see if your socketCAN setup is correct (if not: run `just setup-can`)
async fn test_oze_canopen() -> Result<(), TestError> {
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

    info!("Testing download");
    let dat = s
        .lock()
        .await
        .upload(0x607A, 0)
        .await
        .map_err(TestError::CANOpenError)?;

    info!("Test before download - target position: {dat:?} - Setting to 4x 0x0");

    s.lock()
        .await
        .download(0x607A, 0, &[0x00, 0x00, 0x00, 0x00])
        .await
        .map_err(TestError::CANOpenError)?;
    let dat = s
        .lock()
        .await
        .upload(0x607A, 0)
        .await
        .map_err(TestError::CANOpenError)?;
    info!("Test before download - target position: {dat:?} - Setting to [0x1, 0x2, 0x3, 0x4]");

    s.lock()
        .await
        .download(0x607A, 0, &[0x1, 0x2, 0x3, 0x4])
        .await
        .map_err(TestError::CANOpenError)?;
    let dat = s
        .lock()
        .await
        .upload(0x607A, 0)
        .await
        .map_err(TestError::CANOpenError)?;
    info!("Test after download - target position: {dat:?}");

    info!("Test Statusword upload");
    let dat = s
        .lock()
        .await
        .upload(STATUS_WORD.index, STATUS_WORD.sub_index)
        .await
        .map_err(TestError::CANOpenError)?;

    let sw =
        u16::from_le_bytes(dat[..2].try_into().map_err(|e| {
            TestError::ConversionError(format!("Unable to convert {dat:?} into u16"))
        })?);
    let sw = StatusWord::from_bits(sw).ok_or(TestError::ConversionError(format!(
        "Unable to convert {sw} into statusword"
    )))?;

    info!("Current Statusword: {sw:?}");

    let dat = s
        .lock()
        .await
        .upload(0x1001, 0x0)
        .await
        .map_err(TestError::CANOpenError)?;
    let error: u8 = dat[0];
    info!("Error Register: {error}");

    info!("Attempting Cia402 Transitions");
    const CW: u16 = 0x6040;

    info!("Transition to ReadyToSwitchOn");
    let val = (1u16 << 1) | (1u16 << 2);
    let val = val.to_le_bytes();
    s.lock()
        .await
        .download(CW, 0, &[val[0], val[1]])
        .await
        .map_err(TestError::CANOpenError)?;

    info!("Transition to SwitchedOn");
    let val = (1u16 << 0) | (1u16 << 1) | (1u16 << 2);
    let val = val.to_le_bytes();
    s.lock()
        .await
        .download(CW, 0, &[val[0], val[1]])
        .await
        .map_err(TestError::CANOpenError)?;

    info!("Transition to Operation Enabled");
    let val = (1u16 << 0) | (1u16 << 1) | (1u16 << 2) | (1u16 << 3);
    let val = val.to_le_bytes();
    s.lock()
        .await
        .download(CW, 0, &[val[0], val[1]])
        .await
        .map_err(TestError::CANOpenError)?;
    // stop tasks
    handles.close_and_join().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic_canopen_test() -> Result<(), TestError> {
        gantry_demo::setup_tracing();

        test_oze_canopen().await?;

        Ok(())
    }
}

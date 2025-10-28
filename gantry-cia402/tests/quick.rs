mod common;

use std::time::Duration;

use gantry_cia402::od::{MAX_CURRENT, STATUS_WORD, TORQUE_SLOPE, *};
use oze_canopen::proto::nmt::{NmtCommand, NmtCommandSpecifier};
use tokio::time::sleep;
use tracing::*;

use crate::common::TestError;

const NODE_ID: u8 = 4;

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
    let s = interface.get_sdo_client(NODE_ID).unwrap();

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

    let dat = s
        .lock()
        .await
        .upload(0x60E9, 1)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u16::from_le_bytes([dat[0], dat[1]]);
    info!("Device feed constant: {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(0x60E9, 2)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u16::from_le_bytes([dat[0], dat[1]]);
    info!("Device feed constant: {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(0x060EE, 1)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u16::from_le_bytes([dat[0], dat[1]]);
    info!("Device driving shaft revolutions: {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(0x060EE, 2)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u16::from_le_bytes([dat[0], dat[1]]);
    info!("Device driving shaft revolutions: {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(0x6092, 1)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u16::from_le_bytes([dat[0], dat[1]]);
    info!("Device feed constant 6092h: {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(0x6092, 2)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u16::from_le_bytes([dat[0], dat[1]]);
    info!("Device feed constant 6092h: {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(0x60E8, 1)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u16::from_le_bytes([dat[0], dat[1]]);
    info!("Device motor shaft revolutions {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(0x60E8, 2)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u16::from_le_bytes([dat[0], dat[1]]);
    info!("Device motor shaft revolutions {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(0x60ED, 1)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u16::from_le_bytes([dat[0], dat[1]]);
    info!("Device Driving Shaft Revolutions {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(0x60ED, 2)
        .await
        .map_err(TestError::CANOpenError)?;
    let val = u16::from_le_bytes([dat[0], dat[1]]);
    info!("Device Driving Shaft Revolutions {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(
            LIMIT_SWITCH_OPTION_CODE.index,
            LIMIT_SWITCH_OPTION_CODE.sub_index,
        )
        .await
        .map_err(TestError::CANOpenError)?;
    let val = i16::from_le_bytes([dat[0], dat[1]]);
    info!("Limit switch option code: {}={:#x}", val, val);

    info!("Forgetting limit switch values");

    s.lock()
        .await
        .download(0x607A, 0, &[0x00, 0x00, 0x00, 0x00])
        .await
        .map_err(TestError::CANOpenError)?;

    let data = (-1i16).to_le_bytes(); // forgetting limit switch val = -2, but device doesnt agree :(
    let dat = s
        .lock()
        .await
        .download(
            LIMIT_SWITCH_OPTION_CODE.index,
            LIMIT_SWITCH_OPTION_CODE.sub_index,
            &data,
        )
        .await
        .map_err(TestError::CANOpenError)?;

    let dat = s
        .lock()
        .await
        .upload(
            LIMIT_SWITCH_OPTION_CODE.index,
            LIMIT_SWITCH_OPTION_CODE.sub_index,
        )
        .await
        .map_err(TestError::CANOpenError)?;
    let val = i16::from_le_bytes([dat[0], dat[1]]);
    info!("Limit switch option code: {}={:#x}", val, val);

    let dat = s
        .lock()
        .await
        .upload(
            LIMIT_SWITCH_OPTION_CODE.index,
            LIMIT_SWITCH_OPTION_CODE.sub_index,
        )
        .await
        .map_err(TestError::CANOpenError)?;
    let val = i16::from_le_bytes([dat[0], dat[1]]);
    info!("Limit switch option code: {}={:#x}", val, val);

    let data = (0b000_0000_0000_0001u32).to_le_bytes();
    let dat = s
        .lock()
        .await
        .download(
            DIGITAL_INPUTS_CONTROL_INVERTED.index,
            DIGITAL_INPUTS_CONTROL_INVERTED.sub_index,
            &data,
        )
        .await
        .map_err(TestError::CANOpenError)?;

    let dat = s
        .lock()
        .await
        .upload(
            LIMIT_SWITCH_OPTION_CODE.index,
            LIMIT_SWITCH_OPTION_CODE.sub_index,
        )
        .await
        .map_err(TestError::CANOpenError)?;
    let val = i16::from_le_bytes([dat[0], dat[1]]);
    info!("Limit switch option code: {}={:#x}", val, val);

    let data = (0b000_0000_0000_0011u32).to_le_bytes();
    let dat = s
        .lock()
        .await
        .download(
            DIGITAL_INPUTS_CONTROL_SPECIAL_FUNCTION.index,
            DIGITAL_INPUTS_CONTROL_SPECIAL_FUNCTION.sub_index,
            &data,
        )
        .await
        .map_err(TestError::CANOpenError)?;
    info!(
        "Enabled Negative + Positive limit switch: {}={:#x}",
        val, val
    );

    let data = (0b000_0000_0000_0001u32).to_le_bytes();
    let dat = s
        .lock()
        .await
        .download(
            DIGITAL_INPUTS_ROUTING_ENABLE.index,
            DIGITAL_INPUTS_ROUTING_ENABLE.sub_index,
            &data,
        )
        .await
        .map_err(TestError::CANOpenError)?;
    info!("Enabled DI routing");

    // let data = (1u8).to_le_bytes();
    // let dat = s
    //     .lock()
    //     .await
    //     .download(
    //         DIGITAL_INPUTS_ROUTING_1.index,
    //         DIGITAL_INPUTS_ROUTING_1.sub_index,
    //         &data,
    //     )
    //     .await
    //     .map_err(TestError::CANOpenError)?;
    // info!("Routed physical input 1 to DI 1");

    let data = (1u8).to_le_bytes();
    let dat = s
        .lock()
        .await
        .download(
            DIGITAL_INPUTS_ROUTING_2.index,
            DIGITAL_INPUTS_ROUTING_2.sub_index,
            &data,
        )
        .await
        .map_err(TestError::CANOpenError)?;
    info!("Routed physical input 1 to DI 2");

    // let data = (1u8).to_le_bytes();
    // let dat = s
    //     .lock()
    //     .await
    //     .download(
    //         DIGITAL_INPUTS_ROUTING_3.index,
    //         DIGITAL_INPUTS_ROUTING_3.sub_index,
    //         &data,
    //     )
    //     .await
    //     .map_err(TestError::CANOpenError)?;
    // info!("Routed physical input 1 to DI 3");

    for num in 1..=1000 {
        let dat = s
            .lock()
            .await
            .upload(DIGITAL_INPUTS.index, DIGITAL_INPUTS.sub_index)
            .await
            .map_err(TestError::CANOpenError)?;
        let sf = u16::from_le_bytes([dat[0], dat[1]]);
        let val = u16::from_le_bytes([dat[2], dat[3]]);
        info!("Digital IO - special functions: {:b} - val: {:b}", sf, val);

        let dat = s
            .lock()
            .await
            .upload(
                DIGITAL_INPUTS_RAW_VALUE.index,
                DIGITAL_INPUTS_RAW_VALUE.sub_index,
            )
            .await
            .map_err(TestError::CANOpenError)?;
        let val = u32::from_le_bytes([dat[0], dat[1], dat[2], dat[3]]);
        info!("Raw Digital input values: {:b}", val);

        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    // stop tasks
    handles.close_and_join().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use tokio::signal;

    use super::*;

    #[tokio::test]
    async fn quick_test() -> Result<(), TestError> {
        gantry_demo::setup_tracing();

        tokio::select! {
            res = quick_test_logic() => {
                res?;
            }
            _ = signal::ctrl_c() => {
                info!("Ctrl-C received — aborting test");
            }
        }

        Ok(())
    }
}

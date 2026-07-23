pub mod common;

use std::sync::Arc;
use std::time::Duration;
use tracing::*;

#[cfg(test)]
mod tests {

    use gantry_axis::sync::SyncMaster;
    use gantry_cia402::{
        driver::{oms::OperationMode, receiver::StatusWord},
        od::*,
    };
    use oze_canopen::{
        proto::nmt::{NmtCommand, NmtCommandSpecifier},
        sdo_client::SdoClient,
    };
    use tokio::sync::Mutex;

    use super::*;

    #[tokio::test]
    /// Test basic cia402 state transitions using PDO
    async fn perform_auto_setup() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        pub const NODE_ID: u8 = 1;
        let _node_id = NODE_ID;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        let sync_master = SyncMaster::init(canopen.clone());
        let _sync_rx = sync_master.get_sync_receiver();

        // Motor boots into NMT::PreOperational -> Set motor to NMT::Operational
        canopen
            .send_nmt(NmtCommand::new(
                NmtCommandSpecifier::StartRemoteNode,
                NODE_ID,
            ))
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        // Give the slave device some time to boot, we all have trouble getting out of bed sometimes
        tokio::time::sleep(Duration::from_millis(500)).await;

        info!("Getting sdo client");
        let s = canopen.get_sdo_client(NODE_ID).unwrap();

        info!("Testing upload");
        let dat = s
            .lock()
            .await
            .upload(0x1000, 0)
            .await
            .map_err(|_| anyhow::anyhow!("Upload test failed, is device connected?"))?;

        info!("Test upload - device type: {dat:?}");

        info!("Setting Operational Mode to -2 -> Auto setup");
        let mode = -2i8;
        let mode = mode.to_le_bytes();
        s.lock()
            .await
            .download(
                SET_OPERATION_MODE.index,
                SET_OPERATION_MODE.sub_index,
                &[mode[0]],
            )
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let dat = s
            .lock()
            .await
            .upload(GET_OPERATION_MODE.index, GET_OPERATION_MODE.sub_index)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let mode: i8 = dat[0] as i8;
        let opmode = mode;
        let opmode: OperationMode = opmode
            .try_into()
            .expect("Unable to map {mode} into OperationMode");
        info!("Device in OperationMode {mode} - {opmode:?}");

        let sw = get_statusword(&s).await?;
        info!("Current Statusword: {sw:?}");

        info!("Attempting Cia402 Transitions");
        const CW: u16 = 0x6040;

        info!("Transition to ReadyToSwitchOn");
        let val = (1u16 << 1) | (1u16 << 2);
        let val = val.to_le_bytes();
        s.lock()
            .await
            .download(CW, 0, &[val[0], val[1]])
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let sw = get_statusword(&s).await?;
        info!("Current Statusword: {sw:?}");

        info!("Transition to SwitchedOn");
        let val = (1u16 << 0) | (1u16 << 1) | (1u16 << 2);
        let val = val.to_le_bytes();
        s.lock()
            .await
            .download(CW, 0, &[val[0], val[1]])
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let sw = get_statusword(&s).await?;
        info!("Current Statusword: {sw:?}");

        info!("Transition to Operation Enabled");
        let val = (1u16 << 0) | (1u16 << 1) | (1u16 << 2) | (1u16 << 3);
        let val = val.to_le_bytes();
        s.lock()
            .await
            .download(CW, 0, &[val[0], val[1]])
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let sw = get_statusword(&s).await?;
        info!("Current Statusword: {sw:?}");

        info!("Enable Auto setup");
        let val = (1u16 << 0) | (1u16 << 1) | (1u16 << 2) | (1u16 << 3) | (1u16 << 4);
        let val = val.to_le_bytes();
        s.lock()
            .await
            .download(CW, 0, &[val[0], val[1]])
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        loop {
            info!("Checking if setup is done");
            let sw = get_statusword(&s).await?;
            info!("Current Statusword: {sw:?}");

            if sw.contains(StatusWord::OMS_1) {
                info!("Auto setup is done!");
                break;
            } else {
                info!("Auto setup not done yet :(");
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }

    async fn get_statusword(s: &Arc<Mutex<SdoClient>>) -> anyhow::Result<StatusWord> {
        let dat = s
            .lock()
            .await
            .upload(STATUS_WORD.index, STATUS_WORD.sub_index)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let sw = u16::from_le_bytes(dat[..2].try_into().map_err(|e| anyhow::anyhow!("{e:?}"))?);
        let sw = StatusWord::from_bits(sw)
            .ok_or(anyhow::anyhow!("Unable to convert {sw} into statusword"))?;

        Ok(sw)
    }
}

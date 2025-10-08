pub mod common;

use std::time::Duration;

use gantry_cia402::{
    comms::sdo::SdoAction,
    driver::{
        event::MotorEvent,
        nmt::{Nmt, NmtState},
    },
    od::DEVICE_TYPE,
};
use tokio::task::{self};
use tracing::*;

const NODE_ID: u8 = 3;

const PARAMS: [SdoAction; 1] = [SdoAction::Upload {
    entry: &DEVICE_TYPE,
}];

const TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(test)]
mod tests {

    use gantry_cia402::{
        comms::pdo::mapping::custom::CUSTOM_TPDOS,
        log::{log_canopen_pretty, log_events},
    };

    use common::wait_for_event;

    use crate::common::start_feedback_task;

    use super::*;

    #[tokio::test]
    async fn nmt_boot_test() -> Result<(), String> {
        gantry_demo::setup_tracing();

        let node_id = NODE_ID;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        info!("Starting CANOpen sniffer");
        task::spawn(log_canopen_pretty(canopen.clone()));

        let tpdo_mapping_set = CUSTOM_TPDOS;

        info!("Starting CANOpen event logger");
        let (_, mut event_rx) = start_feedback_task(canopen.clone(), node_id, tpdo_mapping_set);
        task::spawn(log_events(event_rx.resubscribe(), node_id));

        tokio::time::sleep(Duration::from_millis(250)).await;

        info!("Starting NMT handler logger");
        let (nmt_tx, nmt_rx) = tokio::sync::mpsc::channel(10);
        let nmt_handle = Nmt::start(node_id, canopen.clone(), nmt_rx, event_rx.resubscribe());

        nmt_tx
            .send(NmtState::Operational)
            .await
            .map_err(|err| format!("Error requesting NMT state: {err}").to_string())?;

        // Watch for NmtState::Operational
        wait_for_event(
            event_rx.resubscribe(),
            MotorEvent::NmtStateUpdate(NmtState::Operational),
            TIMEOUT,
        )
        .await?;

        tokio::time::sleep(Duration::from_millis(250)).await;

        Ok(())
    }
}

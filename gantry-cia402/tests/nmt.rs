pub mod common;

use std::time::Duration;

use gantry_cia402::driver::{event::MotorEvent, nmt::NmtState};
use tokio::task::{self};
use tracing::*;

#[cfg(test)]
mod tests {

    use gantry_cia402::{
        comms::pdo::mapping::custom::CUSTOM_PDOS,
        driver::{nmt::nmt_task, receiver::subscriber::wait_for_event},
        log::{log_canopen_pretty, log_events},
    };

    use crate::common::{COMMS_TIMEOUT, start_feedback_task};

    use super::*;

    #[tokio::test]
    async fn nmt_boot_test() -> Result<(), String> {
        gantry_demo::setup_tracing();

        let identifier = common::TEST_MOTOR;
        let node_id = identifier.node_id;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        info!("Starting CANOpen sniffer");
        task::spawn(log_canopen_pretty(canopen.clone()));

        info!("Starting CANOpen event logger");
        let (_, event_rx) = start_feedback_task(canopen.clone(), node_id, CUSTOM_PDOS.tpdos);
        task::spawn(log_events(event_rx.resubscribe(), node_id));

        tokio::time::sleep(Duration::from_millis(250)).await;

        let (nmt_tx, nmt_rx) = tokio::sync::mpsc::channel(10);
        // Start the NMT task
        info!("Starting NMT State Machine task for motor with node id {node_id}");
        task::spawn(nmt_task(
            node_id,
            canopen.clone(),
            nmt_rx,
            event_rx.resubscribe(),
        ));

        // Switch to PreOp
        nmt_tx.send(NmtState::PreOperational).await.map_err(|err| {
            format!("Error requesting NMT state PreOperational: {err}").to_string()
        })?;
        wait_for_event(
            event_rx.resubscribe(),
            MotorEvent::NmtStateUpdate(NmtState::PreOperational),
            COMMS_TIMEOUT,
        )
        .await
        .map_err(|err| {
            format!("Error waiting for NmtState::PreOperational: {err:?}").to_string()
        })?;

        // Check if we can switch to Operational
        nmt_tx
            .send(NmtState::Operational)
            .await
            .map_err(|err| format!("Error requesting NMT state Operational: {err}").to_string())?;
        wait_for_event(
            event_rx.resubscribe(),
            MotorEvent::NmtStateUpdate(NmtState::Operational),
            COMMS_TIMEOUT,
        )
        .await
        .map_err(|err| format!("Error waiting for NmtState::Operational: {err:?}").to_string())?;

        tokio::time::sleep(Duration::from_millis(250)).await;

        Ok(())
    }
}

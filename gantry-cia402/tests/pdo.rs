pub mod common;

use std::time::Duration;

use gantry_cia402::driver::{event::MotorEvent, nmt::NmtState};
use tokio::task::{self};
use tracing::*;

#[cfg(test)]
mod tests {

    use gantry_cia402::{
        comms::pdo::mapping::custom::CUSTOM_TPDOS,
        driver::{
            nmt::nmt_task,
            receiver::subscriber::wait_for_event,
            startup::{
                parametrise::parametrise_motor, params::PARAMS, pdo_mapping::configure_pdo_mappings,
            },
        },
        log::log_events,
    };

    use crate::common::*;

    use super::*;

    #[tokio::test]
    async fn configure_pdo_test() -> Result<(), String> {
        gantry_demo::setup_tracing();

        let node_id = NODE_ID;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        // info!("Starting CANOpen sniffer");
        // task::spawn(log_canopen_pretty(canopen.clone()));

        let tpdo_mapping_set = CUSTOM_TPDOS;

        info!("Starting CANOpen event logger");
        let (_, event_rx) = start_feedback_task(canopen.clone(), node_id, tpdo_mapping_set);
        task::spawn(log_events(event_rx.resubscribe(), node_id));

        // Ghetto synchronisation to make sure event logger is up
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

        info!("Requesting NMT Pre-Operational");
        nmt_tx
            .send(NmtState::PreOperational)
            .await
            .map_err(|err| format!("Error requesting NMT state: {err}").to_string())?;

        info!("Wait for NmtState::Pre-Operational");
        wait_for_event(
            event_rx.resubscribe(),
            MotorEvent::NmtStateUpdate(NmtState::PreOperational),
            TIMEOUT,
        )
        .await
        .map_err(|err| {
            format!("Error waiting for NmtState::PreOperational: {err:?}").to_string()
        })?;

        info!("Starting SDO client");
        // Get the SDO client for this node id, we use this to make SDO read/writes
        let sdo = canopen
            .get_sdo_client(node_id)
            .unwrap_or_else(|| panic!("Unable to construct SDO client for node id {node_id}"));

        info!("Starting Parametrisation of motor at node id {node_id}");
        parametrise_motor(node_id, PARAMS, sdo.clone())
            .await
            .map_err(|err| format!("Error during motor parametrisation: {err}").to_string())?;

        info!("Configuring RPDO_mapping of motor at node id {node_id}");
        configure_pdo_mappings(node_id, sdo.clone(), RPDOS)
            .await
            .map_err(|err| format!("Error during RPDO mapping configuration: {err}").to_string())?;

        warn!("Configuring TPDO_mapping of motor at node id {node_id}");
        configure_pdo_mappings(node_id, sdo.clone(), TPDOS)
            .await
            .map_err(|err| format!("Error during TPDO mapping configuration: {err}").to_string())?;

        info!("Requesting NMT Operational");
        nmt_tx
            .send(NmtState::Operational)
            .await
            .map_err(|err| format!("Error requesting NMT state: {err}").to_string())?;

        info!("Wait for NmtState::Operational");
        wait_for_event(
            event_rx.resubscribe(),
            MotorEvent::NmtStateUpdate(NmtState::Operational),
            TIMEOUT,
        )
        .await
        .map_err(|err| format!("Error waiting for NmtState::Operational: {err:?}").to_string())?;

        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok(())
    }
}

pub mod common;

use std::time::Duration;

use gantry_cia402::{
    comms::{
        pdo::mapping::{
            PdoMapping,
            custom::{CUSTOM_RPDOS, CUSTOM_TPDOS},
        },
        sdo::SdoAction,
    },
    driver::{
        event::MotorEvent,
        nmt::{Nmt, NmtState},
        receiver::subscriber::handle_feedback,
    },
    od::DEVICE_TYPE,
};
use oze_canopen::interface::CanOpenInterface;
use tokio::{
    sync::broadcast,
    task::{self, JoinHandle},
};
use tracing::*;

const NODE_ID: u8 = 3;

const TIMEOUT: Duration = Duration::from_secs(2);

const RPDO_MAPPING: &[PdoMapping; 4] = CUSTOM_RPDOS;
const TPDO_MAPPING: &[PdoMapping; 3] = CUSTOM_TPDOS;

/// Start the device feedback task responsible for receiving and parsing device feedback and broadcasting these as events
fn start_feedback_task(
    canopen: CanOpenInterface,
    node_id: u8,
    tpdo_mapping_set: &'static [PdoMapping],
) -> (JoinHandle<()>, broadcast::Receiver<MotorEvent>) {
    // Initialize output interfaces
    let (event_tx, event_rx): (
        broadcast::Sender<MotorEvent>,
        broadcast::Receiver<MotorEvent>,
    ) = tokio::sync::broadcast::channel(10);

    trace!("Starting device feedback handler for motor with node id {node_id}");
    (
        task::spawn(handle_feedback(
            node_id,
            canopen,
            tpdo_mapping_set,
            event_tx,
        )),
        event_rx,
    )
}

#[cfg(test)]
mod tests {

    use gantry_cia402::{
        comms::pdo::mapping::custom::CUSTOM_TPDOS,
        driver::startup::{
            PARAMETRISATION_RETRY_DURATION, parametrise::parametrise_motor, params::PARAMS,
            pdo_mapping::configure_pdo_mappings,
        },
        log::{log_canopen_pretty, log_events},
    };

    use common::wait_for_event;
    use tokio::time::sleep;

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

        info!("Start SDO client");
        // Get the SDO client for this node id, we use this to make SDO read/writes
        let sdo = canopen
            .get_sdo_client(node_id)
            .unwrap_or_else(|| panic!("Unable to construct SDO client for node id {node_id}"));

        tokio::time::sleep(Duration::from_millis(250)).await;

        info!("Configuring RPDO_mapping of motor at node id {node_id}");
        configure_pdo_mappings(node_id, sdo.clone(), RPDO_MAPPING)
            .await
            .map_err(|err| format!("Error during RPDO mapping configuration: {err}").to_string())?;

        info!("Starting parametrisation of motor at node id {node_id}");
        parametrise_motor(node_id, PARAMS, sdo.clone())
            .await
            .map_err(|err| format!("Error during motor parametrisation: {err}").to_string())?;

        info!("Starting NMT handler logger");
        let nmt_handle = Nmt::start(node_id, canopen.clone(), event_rx.resubscribe());

        info!("Wait for NmtState::Operational");
        wait_for_event(
            event_rx.resubscribe(),
            MotorEvent::NmtStateUpdate(NmtState::Operational),
            TIMEOUT,
        )
        .await?;

        Ok(())
    }
}

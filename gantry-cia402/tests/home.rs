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
    driver::{event::MotorEvent, nmt::NmtState, receiver::subscriber::handle_feedback},
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
const TPDO_MAPPING: &[PdoMapping; 4] = CUSTOM_TPDOS;

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
        comms::pdo::{Pdo, mapping::custom::CUSTOM_TPDOS},
        driver::{
            Cia402Driver,
            command::MotorCommand,
            oms::OmsHandler,
            router::command_router,
            startup::{
                RETRY_DURATION, parametrise::parametrise_motor, params::PARAMS,
                pdo_mapping::configure_pdo_mappings,
            },
            state::{Cia402StateMachine, cia402_task},
            update::publisher::publish_updates,
        },
        log::{log_canopen_pretty, log_events},
    };

    use common::wait_for_event;
    use oze_canopen::sync;
    use tokio::{sync::mpsc, time::sleep};

    use crate::common::{RPDOS, TPDOS};

    use super::*;

    #[tokio::test]
    async fn home_test() -> Result<(), String> {
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
        tokio::time::sleep(Duration::from_millis(100)).await;

        let drive = Cia402Driver::init(node_id, canopen, PARAMS, RPDOS, TPDOS).await?;

        Ok(())
    }
}

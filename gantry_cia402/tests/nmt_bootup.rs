pub mod common;

use std::time::Duration;

use gantry_cia402::{
    comms::{pdo::mapping::PdoMapping, sdo::SdoAction},
    driver::{
        event::MotorEvent,
        feedback::receiver::handle_feedback,
        nmt::{Nmt, NmtState},
    },
    od::ObjectDictionary,
};
use oze_canopen::interface::CanOpenInterface;
use tokio::{
    sync::broadcast,
    task::{self, JoinHandle},
};
use tracing::*;

const NODE_ID: u8 = 3;

const PARAMS: [SdoAction; 1] = [SdoAction::Upload {
    index: ObjectDictionary::DEVICE_TYPE.index,
    subindex: ObjectDictionary::DEVICE_TYPE.sub_index,
}];

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

    use gantry_cia402::{log_canopen, log_events};

    use common::wait_for_event;

    use super::*;

    #[tokio::test]
    async fn nmt_boot_test() -> Result<(), String> {
        common::setup_tracing();

        let node_id = NODE_ID;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        let tpdo_mapping_set = PdoMapping::CUSTOM_TPDOS;

        let (_, mut event_rx) = start_feedback_task(canopen.clone(), node_id, tpdo_mapping_set);

        task::spawn(log_events(event_rx.resubscribe(), node_id));
        task::spawn(log_canopen(canopen.clone()));

        let nmt_handle = Nmt::start(node_id, canopen.clone(), event_rx.resubscribe());

        let timeout = Duration::from_secs(15);
        let watch_for = MotorEvent::NmtStateUpdate(NmtState::Operational);
        // TODO also wait for statusword update, or should we just refactor that and parse the
        // statusword from within the feedback task? maybe thats better!
        wait_for_event(event_rx.resubscribe(), watch_for, timeout).await
    }
}

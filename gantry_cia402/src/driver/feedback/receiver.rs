use oze_canopen::{canopen::RxMessage, interface::CanOpenInterface};
use tokio::{
    sync::{broadcast, mpsc, watch},
    time::Instant,
};
use tracing::*;

use crate::{
    comms::pdo::mapping::{PdoMapping, PdoMappingSource},
    driver::{event::MotorEvent, feedback::*, oms::OperationMode, state::Cia402State},
};

pub async fn handle_feedback(
    node_id: u8,
    mut canopen: CanOpenInterface,
    tpdo_mapping: &'static [PdoMapping],
    event_tx: broadcast::Sender<MotorEvent>,
    state_feedback_tx: mpsc::Sender<Cia402State>,
) {
    let mut last_seen = Instant::now();
    let node_id = node_id as u16;

    if let Ok(frame) = canopen.rx.recv().await {
        match frame.cob_id {
            id if id == COB_ID_SYNC => { /* SYNC */ }
            id if id == COB_ID_TPDO1 + node_id => {
                /* TPDO1 */
                trace!("Received TPDO1: {frame:?}");
                handle_tpdo1(frame, tpdo_mapping[1].mappings, &event_tx).await;
            }
            id if id == COB_ID_RPDO1 + node_id => { /* RPDO1 */ }
            id if id == COB_ID_TPDO2 + node_id => {
                /* TPDO2 */
                trace!("Received TPDO2: {frame:?}");
                handle_tpdo2(frame, tpdo_mapping[2].mappings, &event_tx).await;
            }
            id if id == COB_ID_RPDO2 + node_id => { /* RPDO2 */ }
            id if id == COB_ID_TPDO3 + node_id => { /* TPDO3 */ }
            id if id == COB_ID_RPDO3 + node_id => { /* RPDO3 */ }
            id if id == COB_ID_TPDO4 + node_id => { /* TPDO4 */ }
            id if id == COB_ID_RPDO4 + node_id => { /* RPDO4 */ }
            id if id == COB_ID_SDO_TX + node_id => { /* SDO response */ }
            id if id == COB_ID_SDO_RX + node_id => { /* SDO request */ }
            id if id == COB_ID_HEARTBEAT + node_id => {
                /* Heartbeat */
                last_seen = Instant::now();
            }
            _ => { /* ignore */ }
        }
    }

    if Instant::now() - last_seen > COMMS_TIMEOUT {
        event_tx.send(MotorEvent::CommunicationLost);
    }
}

async fn handle_tpdo1(
    frame: RxMessage,
    tpdo1_mappings: &'static [PdoMappingSource],
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    assert!(frame.dlc == 3);

    // 1. Decode statusword
    let statusword = read_statusword(&frame);

    match event_tx.send(MotorEvent::StatusWord(statusword)) {
        Ok(num_subscribers) => {
            info!(
                "Succesfully sent statusword update {statusword:?} to {num_subscribers} subscribers"
            )
        }
        Err(err) => {
            error!("Error sending statusword update: {err}");
        }
    }

    // 2. Decode actual mode of operation
    let raw_mode = frame.data[3] as i8;
    let mode = OperationMode::try_from(raw_mode).expect(&format!(
        "Unable to decode raw_mode: {} into operationmode in handle_tpdo1",
        raw_mode
    ));

    match event_tx.send(MotorEvent::OperationMode(mode)) {
        Ok(num_subscribers) => {
            info!(
                "Succesfully sent operational mode update {mode:?} to {num_subscribers} subscribers"
            )
        }
        Err(err) => {
            error!("Error sending operational mode update: {err}");
        }
    }
}

async fn handle_tpdo2(
    frame: RxMessage,
    tpdo2_mappings: &'static [PdoMappingSource],
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    assert!(frame.dlc == 3);

    // 1. Decode actual position
    let statusword = read_statusword(&frame);

    match event_tx.send(MotorEvent::StatusWord(statusword)) {
        Ok(num_subscribers) => {
            info!(
                "Succesfully sent statusword update {statusword:?} to {num_subscribers} subscribers"
            )
        }
        Err(err) => {
            error!("Error sending statusword update: {err}");
        }
    }

    // 2. Decode actual mode of operation
    let raw_mode = frame.data[3] as i8;
    let mode = OperationMode::try_from(raw_mode).expect(&format!(
        "Unable to decode raw_mode: {} into operationmode in handle_tpdo1",
        raw_mode
    ));

    match event_tx.send(MotorEvent::OperationMode(mode)) {
        Ok(num_subscribers) => {
            info!(
                "Succesfully sent operational mode update {mode:?} to {num_subscribers} subscribers"
            )
        }
        Err(err) => {
            error!("Error sending operational mode update: {err}");
        }
    }
}

fn read_statusword(frame: &RxMessage) -> StatusWord {
    let raw_statusword = u16::from_be_bytes([frame.data[0], frame.data[1]]);
    StatusWord::from_bits(raw_statusword).expect("unable to decode statusword")
}

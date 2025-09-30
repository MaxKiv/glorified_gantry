use std::time::Duration;

use oze_canopen::{canopen::RxMessage, interface::CanOpenInterface};
use tokio::{
    sync::{broadcast, watch},
    time::Instant,
};
use tracing::*;

use crate::{
    comms::pdo::mapping::{PdoMapping, PdoMappingSource, PdoType},
    driver::{event::MotorEvent, state::Cia402State, update::ControlWord},
    od::drive_subscriber::StatusWord,
};

const COB_ID_SYNC: u16 = 0x80;
const COB_ID_TPDO1: u16 = 0x180;
const COB_ID_RPDO1: u16 = 0x200;
const COB_ID_TPDO2: u16 = 0x280;
const COB_ID_RPDO2: u16 = 0x300;
const COB_ID_TPDO3: u16 = 0x380;
const COB_ID_RPDO3: u16 = 0x400;
const COB_ID_TPDO4: u16 = 0x480;
const COB_ID_RPDO4: u16 = 0x500;
const COB_ID_SDO_RX: u16 = 0x600;
const COB_ID_SDO_TX: u16 = 0x580;
const COB_ID_HEARTBEAT: u16 = 0x700;

const COMMS_TIMEOUT: Duration = Duration::from_secs(1);

pub async fn handle_feedback(
    node_id: u8,
    mut canopen: CanOpenInterface,
    tpdo_mapping: &'static [PdoMapping],
    event_tx: broadcast::Sender<MotorEvent>,
    state_tx: watch::Sender<Cia402State>,
) {
    let mut last_seen = Instant::now();
    let node_id = node_id as u16;

    if let Ok(frame) = canopen.rx.recv().await {
        match frame.cob_id {
            id if id == COB_ID_SYNC => { /* SYNC */ }
            id if id == COB_ID_TPDO1 + node_id => {
                /* TPDO1 */
                handle_tpdo1(frame, tpdo_mapping[1].mappings, event_tx, state_tx).await;
            }
            id if id == COB_ID_RPDO1 + node_id => { /* RPDO1 */ }
            id if id == COB_ID_TPDO2 + node_id => { /* TPDO2 */ }
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
    event_tx: broadcast::Sender<MotorEvent>,
    state_tx: watch::Sender<Cia402State>,
) {
    let raw_statusword = u16::from_be_bytes(frame.data[0..=1]);
    let statusword = StatusWord::from_bits(raw_statusword).expect("unable to decode statusword");

    let raw_mode = i8::from_be_bytes(frame.data[3]);
    let mode = OperationMode::try_from(raw)?;
}

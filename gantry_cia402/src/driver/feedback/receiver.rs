use oze_canopen::{canopen::RxMessage, interface::CanOpenInterface};
use tokio::{
    sync::broadcast,
    time::Instant,
};
use tracing::*;

use crate::{
    comms::pdo::mapping::{PdoMapping, PdoMappingSource},
    driver::{event::MotorEvent, feedback::*, oms::OperationMode},
};

pub async fn handle_feedback(
    node_id: u8,
    mut canopen: CanOpenInterface,
    tpdo_mapping: &'static [PdoMapping],
    event_tx: broadcast::Sender<MotorEvent>,
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
            id if id == COB_ID_TPDO3 + node_id => {
                /* TPDO3 */
                trace!("Received TPDO3: {frame:?}");
                handle_tpdo3(frame, tpdo_mapping[3].mappings, &event_tx).await;
            }
            id if id == COB_ID_RPDO3 + node_id => { /* RPDO3 */ }
            id if id == COB_ID_TPDO4 + node_id => { /* TPDO4 */ }
            id if id == COB_ID_RPDO4 + node_id => {
                /* RPDO4 */
                trace!(
                    "Received TPDO4: {frame:?} - This is strange because we are only mapping 3, is the mapping done correctly?"
                );
            }
            id if id == COB_ID_SDO_TX + node_id => { /* SDO response */ }
            id if id == COB_ID_SDO_RX + node_id => { /* SDO request */ }
            id if id == COB_ID_HEARTBEAT + node_id => {
                /* Heartbeat */
                trace!("Received Heartbeat from: {node_id}");
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
    match read_statusword(&frame) {
        Ok(statusword) => match event_tx.send(MotorEvent::StatusWord(statusword)) {
            Ok(num_subscribers) => {
                info!(
                    "Succesfully sent statusword update {:?} to {num_subscribers} subscribers",
                    statusword
                )
            }
            Err(err) => {
                error!("Error sending statusword  update: {err}");
            }
        },
        Err(err) => {
            error!("Error reading statusword from TPDO1: {err}");
        }
    }

    // 2. Decode actual mode of operation
    match read_operational_mode(&frame) {
        Ok(opmode) => match event_tx.send(MotorEvent::OperationMode(opmode)) {
            Ok(num_subscribers) => {
                info!(
                    "Succesfully sent operational mode update {opmode:?} to {num_subscribers} subscribers"
                )
            }
            Err(err) => {
                error!("Error sending operational mode update: {err}");
            }
        },
        Err(err) => {
            error!("Unable to read operational mode from TPDO1: {err}");
        }
    }
}

async fn handle_tpdo2(
    frame: RxMessage,
    tpdo2_mappings: &'static [PdoMappingSource],
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    assert!(frame.dlc == 8);

    match read_actual_position(&frame) {
        Ok(actual_position) => {
            match event_tx.send(MotorEvent::PositionFeedback {
                actual_position: actual_position.value,
            }) {
                Ok(num_subscribers) => {
                    info!(
                        "Succesfully sent actual position update {} to {num_subscribers} subscribers",
                        actual_position.value
                    )
                }
                Err(err) => {
                    error!("Error sending atcual position update: {err}");
                }
            }
        }
        Err(err) => {
            error!("Error reading actual position: {err}");
        }
    }

    match read_actual_velocity(&frame) {
        Ok(actual_velocity) => {
            match event_tx.send(MotorEvent::VelocityFeedback {
                actual_velocity: actual_velocity.value,
            }) {
                Ok(num_subscribers) => {
                    info!(
                        "Succesfully sent actual velocity update {} to {num_subscribers} subscribers",
                        actual_velocity.value
                    )
                }
                Err(err) => {
                    error!("Error sending atcual velocity update: {err}");
                }
            }
        }
        Err(err) => {
            error!("Error reading actual velocity: {err}");
        }
    }
}

async fn handle_tpdo3(
    frame: RxMessage,
    tpdo2_mappings: &'static [PdoMappingSource],
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    assert!(frame.dlc == 2);

    match read_actual_torque(&frame) {
        Ok(actual_torque) => {
            match event_tx.send(MotorEvent::TorqueFeedback {
                actual_torque: actual_torque.value,
            }) {
                Ok(num_subscribers) => {
                    info!(
                        "Succesfully sent actual torque update {} to {num_subscribers} subscribers",
                        actual_torque.value
                    )
                }
                Err(err) => {
                    error!("Error sending atcual torque update: {err}");
                }
            }
        }
        Err(err) => {
            error!("Error reading actual torque: {err}");
        }
    }
}

fn read_statusword(frame: &RxMessage) -> anyhow::Result<StatusWord> {
    // TODO: move range and type info to central place
    const STATUSWORD_BYTES: std::ops::RangeInclusive<usize> = 0..=1;

    let raw_statusword = u16::from_be_bytes(frame.data[STATUSWORD_BYTES].try_into()?);
    Ok(StatusWord::from_bits(raw_statusword).ok_or(anyhow::anyhow!(
        "unable to decode raw statusword: {raw_statusword:?} into u16"
    ))?)
}

fn read_operational_mode(frame: &RxMessage) -> anyhow::Result<OperationMode> {
    // TODO: move range and type info to central place
    const OPMODE_BYTE: usize = 2;

    let raw_mode = frame.data[OPMODE_BYTE] as i8;
    OperationMode::try_from(raw_mode).map_err(|_| {
        anyhow::anyhow!(
            "invalid raw mode: {} while decoding operational mode from TPDO1 (byte {OPMODE_BYTE})",
            raw_mode
        )
    })
}

fn read_actual_position(frame: &RxMessage) -> anyhow::Result<ActualPosition> {
    // TODO: move range and type info to central place
    const ACTUAL_POS_BYTES: std::ops::RangeInclusive<usize> = 0..=3;

    Ok(ActualPosition {
        value: i32::from_be_bytes(frame.data[ACTUAL_POS_BYTES].try_into()?),
    })
}

fn read_actual_velocity(frame: &RxMessage) -> anyhow::Result<ActualVelocity> {
    // TODO: move range and type info to central place
    const ACTUAL_VEL_BYTES: std::ops::RangeInclusive<usize> = 4..=7;

    Ok(ActualVelocity {
        value: i32::from_be_bytes(frame.data[ACTUAL_VEL_BYTES].try_into()?),
    })
}

fn read_actual_torque(frame: &RxMessage) -> anyhow::Result<ActualTorque> {
    // TODO: move range and type info to central place
    const ACTUAL_TORQUE_BYTES: std::ops::RangeInclusive<usize> = 0..=1;

    Ok(ActualTorque {
        value: i16::from_be_bytes(frame.data[ACTUAL_TORQUE_BYTES].try_into()?),
    })
}

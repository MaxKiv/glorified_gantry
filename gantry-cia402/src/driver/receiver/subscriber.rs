use oze_canopen::{canopen::RxMessage, interface::CanOpenInterface};
use tokio::{sync::broadcast, time::Instant};
use tracing::*;

use crate::{
    comms::pdo::mapping::PdoMapping,
    driver::{
        event::MotorEvent,
        nmt::NmtState,
        oms::OperationMode,
        receiver::{
            error::ReceiverError,
            frame::{Frame, MessageType, ParseError},
            *,
        },
    },
    log::format_frame,
};

pub async fn handle_feedback(
    this_node_id: u8,
    mut canopen: CanOpenInterface,
    tpdo_mapping: &'static [PdoMapping],
    event_tx: broadcast::Sender<MotorEvent>,
) {
    let mut last_seen = Instant::now();

    trace!("Starting feedback handling loop");

    loop {
        if let Ok(message) = canopen.rx.recv().await {
            let span = span!(Level::TRACE, "receiver");
            let _enter = span.enter();

            // trace!("Received frame: {}", format_frame(&message));

            // Parse received frames
            let Ok(parsed): Result<Frame, _> = message.try_into() else {
                error!("Error parsing message: {message:?}");
                continue;
            };
            parsed.log();

            // Skip messages that are not from the motor that we are managing
            if parsed
                .node_id
                .is_some_and(|message_id| message_id == this_node_id)
            {
                // Our node talked, you love to see it
                last_seen = Instant::now();

                // Lets check what message we got
                if let Err(err) = handle_message(&parsed.message, &event_tx, &tpdo_mapping).await {
                    error!(
                        "Error while handling this message: {:?} - {err}",
                        parsed.message
                    );
                }
            }

            if Instant::now() - last_seen > COMMS_TIMEOUT
                && let Err(err) = event_tx.send(MotorEvent::CommunicationLost)
            {
                error!("Unable to broadcast CommunicationLost message: {err}");
            }
        }
    }
}

async fn handle_message(
    message: &MessageType,
    event_tx: &broadcast::Sender<MotorEvent>,
    tpdo_mapping: &&'static [PdoMapping],
) -> Result<(), ReceiverError> {
    match message {
        MessageType::NmtControl(_) => {
            // We sent this: Ignore
        }
        MessageType::EMCY(emergency_message) => {
            handle_emcy(emergency_message, &event_tx).await;
        }
        MessageType::TSDO(sdo_response) => {
            handle_sdo_response(sdo_response, &event_tx).await;
        }
        MessageType::RSDO(_) => {
            // We sent this: Ignore
        }
        MessageType::TPDO(tpdomessage) => {
            handle_tpdo(tpdomessage, &tpdo_mapping, &event_tx).await?;
        }

        MessageType::RPDO(_) => {
            // We sent this: Ignore
        }
        MessageType::NmtMonitor(nmt_monitor_message) => {
            handle_nmt_monitor(nmt_monitor_message, &event_tx).await;
        }
        // SYNC and UNKNOWN are both not addressed to a single node, we not adress those here: Ignore
        _ => unreachable!(),
    };

    Ok(())
}

async fn handle_sdo_response(
    sdo_response: &frame::sdo_response::SdoResponse,
    event_tx: &&broadcast::Sender<MotorEvent>,
) {
    send_update(MotorEvent::SdoResponse(sdo_response.clone()), event_tx);
}

async fn handle_emcy(
    emergency_message: &frame::EmergencyMessage,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    send_update(MotorEvent::EMCY(emergency_message.error.clone()), event_tx);
}

async fn handle_tpdo(
    tpdomessage: &frame::TPDOMessage,
    tpdo_mapping: &'static [PdoMapping],
    event_tx: &broadcast::Sender<MotorEvent>,
) -> Result<(), ReceiverError> {
    match &tpdomessage.num {
        1 => {
            handle_tpdo1(tpdomessage, &tpdo_mapping[0], event_tx).await;
            Ok(())
        }
        2 => {
            handle_tpdo2(tpdomessage, &tpdo_mapping[1], event_tx).await;
            Ok(())
        }
        3 => {
            handle_tpdo3(tpdomessage, &tpdo_mapping[2], event_tx).await;
            Ok(())
        }
        _ => {
            // Err(ReceiverError::UnknownTPDO(tpdomessage.clone())),
            warn!("Received unknown / unmapped RPDO: {:?}", tpdomessage);
            Ok(())
        }
    }
}

async fn handle_nmt_monitor(
    nmt_monitor_message: &frame::NmtMonitorMessage,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    send_update(
        MotorEvent::NmtStateUpdate(nmt_monitor_message.current_state.clone()),
        event_tx,
    );
}

async fn handle_tpdo1(
    tpdomessage: &frame::TPDOMessage,
    tpdo1_mapping: &PdoMapping,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    assert!(tpdomessage.dlc == 3);

    // 1. Decode statusword
    match read_statusword(&tpdomessage.data) {
        Ok(statusword) => {
            // Parse NMT bits
            let new_nmt_state: NmtState = statusword.into();

            // Send fresh NMT state to subscribers
            send_update(MotorEvent::NmtStateUpdate(new_nmt_state), event_tx);

            // Send rest of statusword to subscribers
            send_update(MotorEvent::StatusWord(statusword), event_tx);
        }
        Err(err) => {
            error!("Error reading statusword from TPDO1: {err}");
        }
    }

    // 2. Decode actual mode of operation
    match read_operational_mode(&tpdomessage.data) {
        Ok(opmode) => {
            // Send operational mode update
            send_update(MotorEvent::OperationMode(opmode), event_tx);
        }
        Err(err) => {
            error!("Unable to read operational mode from TPDO1: {err}");
        }
    }
}

async fn handle_tpdo2(
    tpdomessage: &frame::TPDOMessage,
    tpdo2_mappings: &PdoMapping,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    assert!(tpdomessage.dlc == 8);

    match read_actual_position(&tpdomessage.data) {
        Ok(actual_position) => {
            // Send actual position update
            send_update(
                MotorEvent::PositionFeedback {
                    actual_position: actual_position.value,
                },
                event_tx,
            );
        }
        Err(err) => {
            error!("Error reading actual position: {err}");
        }
    }

    match read_actual_velocity(&tpdomessage.data) {
        Ok(actual_velocity) => {
            // Send actual velocity update
            send_update(
                MotorEvent::VelocityFeedback {
                    actual_velocity: actual_velocity.value,
                },
                event_tx,
            );
        }
        Err(err) => {
            error!("Error reading actual velocity: {err}");
        }
    }
}

async fn handle_tpdo3(
    tpdomessage: &frame::TPDOMessage,
    tpdo2_mappings: &PdoMapping,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    assert!(tpdomessage.dlc == 2);

    match read_actual_torque(&tpdomessage.data) {
        Ok(actual_torque) => {
            // Send actual torque update
            send_update(
                MotorEvent::TorqueFeedback {
                    actual_torque: actual_torque.value,
                },
                event_tx,
            );
        }
        Err(err) => {
            error!("Error reading actual torque: {err}");
        }
    }
}

fn read_statusword(data: &[u8; 8]) -> anyhow::Result<StatusWord> {
    // TODO: move range and type info to central place
    const STATUSWORD_BYTES: std::ops::RangeInclusive<usize> = 0..=1;

    let raw_statusword = u16::from_be_bytes(data[STATUSWORD_BYTES].try_into()?);
    StatusWord::from_bits(raw_statusword).ok_or(anyhow::anyhow!(
        "unable to decode raw statusword: {raw_statusword:?} into u16"
    ))
}

fn read_operational_mode(data: &[u8; 8]) -> anyhow::Result<OperationMode> {
    // TODO: move range and type info to central place
    const OPMODE_BYTE: usize = 2;

    let raw_mode = data[OPMODE_BYTE] as i8;
    OperationMode::try_from(raw_mode).map_err(|_| {
        anyhow::anyhow!(
            "invalid raw mode: {} while decoding operational mode from TPDO1 (byte {OPMODE_BYTE})",
            raw_mode
        )
    })
}

fn read_actual_position(data: &[u8; 8]) -> anyhow::Result<ActualPosition> {
    // TODO: move range and type info to central place
    const ACTUAL_POS_BYTES: std::ops::RangeInclusive<usize> = 0..=3;

    Ok(ActualPosition {
        value: i32::from_be_bytes(data[ACTUAL_POS_BYTES].try_into()?),
    })
}

fn read_actual_velocity(data: &[u8; 8]) -> anyhow::Result<ActualVelocity> {
    // TODO: move range and type info to central place
    const ACTUAL_VEL_BYTES: std::ops::RangeInclusive<usize> = 4..=7;

    Ok(ActualVelocity {
        value: i32::from_be_bytes(data[ACTUAL_VEL_BYTES].try_into()?),
    })
}

fn read_actual_torque(data: &[u8; 8]) -> anyhow::Result<ActualTorque> {
    // TODO: move range and type info to central place
    const ACTUAL_TORQUE_BYTES: std::ops::RangeInclusive<usize> = 0..=1;

    Ok(ActualTorque {
        value: i16::from_be_bytes(data[ACTUAL_TORQUE_BYTES].try_into()?),
    })
}

fn send_update(event: MotorEvent, event_tx: &broadcast::Sender<MotorEvent>) {
    match event_tx.send(event.clone()) {
        Ok(num_subscribers) => {
            info!(
                "Succesfully sent update {:?} to {num_subscribers} subscribers",
                event
            )
        }
        Err(err) => {
            error!("Error sending update: {err}");
        }
    }
}

use oze_canopen::{canopen::RxMessage, interface::CanOpenInterface};
use tokio::{
    sync::broadcast,
    time::{self, Instant},
};
use tracing::*;

use crate::{
    comms::pdo::mapping::PdoMapping,
    driver::{
        event::MotorEvent,
        nmt::NmtState,
        oms::{
            OperationMode, home::HomeFlagsSW, position::PositionFlagsSW, torque::TorqueFlagsSW,
            velocity::VelocityFlagsSW,
        },
        receiver::{
            error::ReceiverError,
            parse::{Frame, MessageType, pdo_message::*},
            *,
        },
        state::{Cia402Flags, Cia402State},
    },
    error::DriveError,
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
        match tokio::time::timeout(Duration::from_secs(2), canopen.clone().rx.recv()).await {
            Ok(Ok(message)) => {
                let span = span!(Level::TRACE, "receiver");
                let _enter = span.enter();

                trace!("Received frame: {}", format_frame(&message));

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
                    trace!("message {message:?} is for this node {this_node_id} - processing");
                    // Our node talked, you love to see it
                    last_seen = Instant::now();

                    // Lets check what message we got
                    if let Err(err) =
                        handle_message(&parsed.message, &event_tx, &tpdo_mapping).await
                    {
                        error!(
                            "Error while handling this message: {:?} - {err}",
                            parsed.message
                        );
                    }
                } else {
                    trace!("message not for node {this_node_id}: {message:?} - skipping")
                }

                if Instant::now() - last_seen > COMMS_TIMEOUT
                    && let Err(err) = event_tx.send(MotorEvent::CommunicationLost)
                {
                    error!("Unable to broadcast CommunicationLost message: {err}");
                }
            }
            Ok(Err(err)) => {
                error!("feedback error: {err}");
            }
            Err(_) => {
                error!(
                    "feedback idle >2s, this might indicate a stalled receiver -> resubscribing"
                );
                canopen.rx = canopen.rx.resubscribe();
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
            handle_emcy(emergency_message, event_tx).await;
        }
        MessageType::TSDO(sdo_response) => {
            handle_sdo_response(sdo_response, event_tx).await;
        }
        MessageType::RSDO(_) => {
            // We sent this: Ignore
        }
        MessageType::TPDO(tpdomessage) => {
            handle_tpdo(tpdomessage, tpdo_mapping, event_tx).await?;
        }
        MessageType::RPDO(_) => {
            // We sent this: Ignore
        }
        MessageType::PDO(parsed_pdo) => {
            handle_parsed_pdo(parsed_pdo, event_tx).await;
        }
        MessageType::NmtMonitor(nmt_monitor_message) => {
            handle_nmt_monitor(nmt_monitor_message, event_tx).await;
        }
        // SYNC and UNKNOWN are both not addressed to a single node, we not adress those here: Ignore
        MessageType::Sync(_) | MessageType::Unknown(_) => {
            // Not for us: Ignore
        }
    };

    Ok(())
}

async fn handle_parsed_pdo(
    parsed_pdo: &parse::pdo_message::ParsedPDO,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    match &parsed_pdo.message {
        parse::pdo_message::PDOMessage::TPDO1(tpdo1_message) => {
            handle_parsed_tpdo1(tpdo1_message, event_tx).await;
        }
        parse::pdo_message::PDOMessage::TPDO2(tpdo2_message) => {
            handle_parsed_tpdo2(tpdo2_message, event_tx).await;
        }
        parse::pdo_message::PDOMessage::TPDO3(tpdo3_message) => {
            handle_parsed_tpdo3(tpdo3_message, event_tx).await;
        }
        parse::pdo_message::PDOMessage::TPDO4(tpdo4_message) => {
            // TPDO4 is unmapped
            warn!(
                "Received TPDO4: {tpdo4_message:?}, however this should be unmapped 🤔, ignoring..."
            );
        }
        parse::pdo_message::PDOMessage::Raw(raw_pdomessage) => {
            warn!("Received weird parsed pdo: {raw_pdomessage:?}, ignoring...");
        }
        _ => {
            // RPDO messages are sent by us, purposfully ignored here
        }
    }
}

async fn handle_sdo_response(
    sdo_response: &parse::sdo_response::SdoResponse,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    send_update(MotorEvent::SdoResponse(sdo_response.clone()), event_tx);
}

async fn handle_emcy(
    emergency_message: &parse::EmergencyMessage,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    send_update(MotorEvent::EMCY(emergency_message.error.clone()), event_tx);
}

async fn handle_tpdo(
    tpdomessage: &parse::TPDOMessage,
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
    nmt_monitor_message: &parse::NmtMonitorMessage,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    send_update(
        MotorEvent::NmtStateUpdate(nmt_monitor_message.current_state.clone()),
        event_tx,
    );
}

async fn handle_parsed_tpdo1(
    tpdo1_message: &TPDO1Message,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    // Parse NMT bits
    let new_nmt_state: NmtState = tpdo1_message.statusword.into();
    // Send fresh NMT state to subscribers
    send_update(MotorEvent::NmtStateUpdate(new_nmt_state), event_tx);

    // Send rest of statusword to subscribers
    send_update(MotorEvent::StatusWord(tpdo1_message.statusword), event_tx);

    // Send operational mode update
    send_update(
        MotorEvent::OperationModeUpdate(tpdo1_message.actual_opmode),
        event_tx,
    );
}

async fn handle_tpdo1(
    tpdomessage: &parse::TPDOMessage,
    tpdo1_mapping: &PdoMapping,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    assert!(tpdomessage.dlc == 3);

    // 1. Decode statusword
    let sw = match read_statusword(&tpdomessage.data) {
        Ok(statusword) => {
            // Parse NMT bits
            let new_nmt_state: NmtState = statusword.into();
            // Send fresh NMT state to subscribers
            send_update(MotorEvent::NmtStateUpdate(new_nmt_state), event_tx);

            // Send rest of statusword to subscribers
            send_update(MotorEvent::StatusWord(statusword), event_tx);

            Some(statusword)
        }
        Err(err) => {
            error!("Error reading statusword from TPDO1: {err}");
            None
        }
    };

    // 2. Decode actual mode of operation
    match read_operational_mode(&tpdomessage.data) {
        Ok(opmode) => {
            // Send operational mode update
            send_update(MotorEvent::OperationModeUpdate(opmode), event_tx);

            // Parse Operational Mode Specific bits
            if let Some(statusword) = sw {
                parse_oms_statusword_bits(opmode, statusword, event_tx);
            }
        }
        Err(err) => {
            error!("Unable to read operational mode from TPDO1: {err}");
        }
    }
}

fn parse_oms_statusword_bits(
    opmode: OperationMode,
    statusword: StatusWord,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    // Parse Operation Mode Specific bits of the statusword
    let event = match opmode {
        OperationMode::ProfilePosition => {
            let flags = PositionFlagsSW::from_status(statusword);
            Some(flags.into_event())
        }
        OperationMode::ProfileVelocity => {
            let flags = VelocityFlagsSW::from_status(statusword);
            Some(flags.into_event())
        }
        OperationMode::ProfileTorque => {
            let flags = TorqueFlagsSW::from_status(statusword);
            Some(flags.into_event())
        }
        OperationMode::Homing => {
            let flags = HomeFlagsSW::from_status(statusword);
            Some(flags.into_event())
        }
        _ => {
            trace!("No specific statusword parsing for current opmode {opmode:?}");
            None
        }
    };

    // Send anything interesting along
    if let Some(event) = event {
        trace!("Sending OMS event: {event:?}");
        send_update(event, event_tx);
    }
}

async fn handle_parsed_tpdo2(
    tpdo2_message: &TPDO2Message,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    // Send actual position update
    send_update(
        MotorEvent::PositionFeedback {
            actual_position: tpdo2_message.actual_pos,
        },
        event_tx,
    );

    // Send actual velocity update
    send_update(
        MotorEvent::VelocityFeedback {
            actual_velocity: tpdo2_message.actual_vel,
        },
        event_tx,
    );
}

async fn handle_tpdo2(
    tpdomessage: &parse::TPDOMessage,
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

async fn handle_parsed_tpdo3(
    tpdo3_message: &TPDO3Message,
    event_tx: &broadcast::Sender<MotorEvent>,
) {
    // Send actual torque update
    send_update(
        MotorEvent::TorqueFeedback {
            actual_torque: tpdo3_message.actual_torque,
        },
        event_tx,
    );
}

async fn handle_tpdo3(
    tpdomessage: &parse::TPDOMessage,
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

    let raw_statusword = u16::from_le_bytes(data[STATUSWORD_BYTES].try_into()?);
    // trace!("Decoding raw statusword: {:#0x}", raw_statusword);
    let sw = StatusWord::from_bits(raw_statusword).ok_or(anyhow::anyhow!(
        "unable to decode raw statusword: {raw_statusword:?} into u16"
    ))?;

    // trace!("Decoded statusword from tpdo1: {sw:?}");
    Ok(sw)
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

pub async fn wait_for_event(
    mut event_rx: broadcast::Receiver<MotorEvent>,
    watch_for: MotorEvent,
    timeout: Duration,
) -> Result<(), DriveError> {
    let deadline = Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            warn!("Timeout when waiting for event: {watch_for:?}");
            return Err(DriveError::EventTimeout(watch_for, None));
        }

        let recv_future = event_rx.recv();
        let result = time::timeout(remaining, recv_future).await;

        match result {
            Ok(Ok(event)) => {
                if event == watch_for {
                    return Ok(());
                }
                // else keep looping for the next one
            }
            Ok(Err(err @ broadcast::error::RecvError::Lagged(_))) => {
                // Messages were missed, continue to next one
                error!("Lagged in wait_for_event, indicates serious issue");
                return Err(DriveError::BroadcastLagged(watch_for, err));
            }
            Ok(Err(err @ broadcast::error::RecvError::Closed)) => {
                error!("Event channel closed in wait_for_event");
                return Err(DriveError::BroadcastClosed(watch_for, err));
            }
            Err(err) => {
                warn!("Timeout when waiting for event: {watch_for:?}");
                return Err(DriveError::EventTimeout(watch_for, Some(err)));
            }
        }
    }
}

use oze_canopen::interface::CanOpenInterface;
use tokio::{
    sync::broadcast,
    time::{self, Instant},
};
use tracing::*;

use crate::{
    comms::pdo::mapping::PdoMapping,
    driver::{
        event::MotorEvent,
        oms::OMSFlagsSW,
        receiver::{
            error::ReceiverError,
            parse::{Frame, MessageType, pdo_message::*},
            *,
        },
    },
    error::DriveError,
    log::format_frame,
};

/// Central task that handles all receiving communications from the motor/device
/// This device feedback is parsed, and relevant information is broadcast as [`MotorEvent`]
pub async fn handle_feedback(
    this_node_id: u8,
    mut canopen: CanOpenInterface,
    tpdo_mapping: &'static [PdoMapping],
    event_tx: broadcast::Sender<MotorEvent>,
) -> Result<(), DriveError> {
    let mut last_seen = Instant::now();

    trace!("Starting feedback handling loop");

    loop {
        match tokio::time::timeout(Duration::from_secs(2), canopen.rx.recv()).await {
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
                    trace!(
                        "message {message:?} - parsed {parsed:?} is for this node {this_node_id} - processing"
                    );
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
                    // trace!("message not for node {this_node_id}: {message:?} - skipping")
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
    // Send full statusword update to subscribers
    send_update(MotorEvent::StatusWord(tpdo1_message.statusword), event_tx);

    // Send operational mode update
    send_update(
        MotorEvent::OperationModeUpdate(tpdo1_message.actual_opmode),
        event_tx,
    );

    // Parse Operational Mode Specific bits
    let event = match tpdo1_message.oms_flags {
        OMSFlagsSW::Homing(home_flags_sw) => Some(home_flags_sw.into_event()),
        OMSFlagsSW::ProfilePosition(position_flags_sw) => Some(position_flags_sw.into_event()),
        OMSFlagsSW::ProfileVelocity(velocity_flags_sw) => Some(velocity_flags_sw.into_event()),
        OMSFlagsSW::ProfileTorque(torque_flags_sw) => Some(torque_flags_sw.into_event()),
        OMSFlagsSW::None => None,
        OMSFlagsSW::CyclicSynchronousPosition(cyclic_pos_flags_sw) => {
            Some(cyclic_pos_flags_sw.into_event())
        }
        OMSFlagsSW::CyclicSynchronousVelocity(cyclic_vel_flags_sw) => {
            Some(cyclic_vel_flags_sw.into_event())
        }
        OMSFlagsSW::CyclicSynchronousTorque(cyclic_torque_flags_sw) => {
            Some(cyclic_torque_flags_sw.into_event())
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
                return Err(DriveError::BroadcastLagged(Some(watch_for), err));
            }
            Ok(Err(err @ broadcast::error::RecvError::Closed)) => {
                error!("Event channel closed in wait_for_event");
                return Err(DriveError::BroadcastClosed(Some(watch_for), err));
            }
            Err(err) => {
                warn!("Timeout when waiting for event: {watch_for:?}");
                return Err(DriveError::EventTimeout(watch_for, Some(err)));
            }
        }
    }
}

pub async fn wait_until_event_matches<F>(
    mut event_rx: broadcast::Receiver<MotorEvent>,
    predicate: F,
    timeout: Duration,
) -> Result<(), DriveError>
where
    F: Fn(&MotorEvent) -> bool,
{
    let deadline = Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            warn!("Timeout while waiting for event");
            return Err(DriveError::EventMatchesTimeout);
        }

        let result = time::timeout(remaining, event_rx.recv()).await;

        match result {
            Ok(Ok(event)) => {
                if predicate(&event) {
                    return Ok(());
                }
            }
            Ok(Err(err @ broadcast::error::RecvError::Lagged(_))) => {
                error!("Lagged in wait_for_event, indicates serious issue");
                return Err(DriveError::BroadcastLagged(None, err));
            }
            Ok(Err(err @ broadcast::error::RecvError::Closed)) => {
                error!("Event channel closed in wait_for_event");
                return Err(DriveError::BroadcastClosed(None, err));
            }
            Err(_) => {
                warn!("Timeout when waiting for event");
                return Err(DriveError::EventMatchesTimeout);
            }
        }
    }
}

pub async fn wait_for_setpoint_acknowledge(
    event_rx: broadcast::Receiver<MotorEvent>,
    timeout: Duration,
) -> Result<(), DriveError> {
    wait_until_event_matches(
        event_rx,
        |event| {
            matches!(
                event,
                MotorEvent::PositionModeFeedback {
                    setpoint_acknowlegded: true,
                    ..
                }
            )
        },
        timeout,
    )
    .await
}

pub async fn wait_for_target_reached(
    event_rx: broadcast::Receiver<MotorEvent>,
    timeout: Duration,
) -> Result<(), DriveError> {
    wait_until_event_matches(
        event_rx,
        |event| {
            matches!(
                event,
                MotorEvent::PositionModeFeedback {
                    target_reached: true,
                    ..
                }
            )
        },
        timeout,
    )
    .await
}

pub mod home;
pub mod parametrise;
pub mod params;
pub mod pdo_mapping;

use std::{sync::Arc, time::Duration};

use oze_canopen::sdo_client::SdoClient;
use tokio::{
    sync::{Mutex, broadcast, mpsc},
    time::{sleep, timeout},
};
use tracing::*;

use crate::{
    comms::{pdo::mapping::PdoMapping, sdo::SdoAction},
    driver::{
        event::MotorEvent,
        nmt::NmtState,
        receiver::subscriber::wait_for_event,
        startup::{parametrise::parametrise_motor, pdo_mapping::configure_pdo_mappings},
    },
    error::DriveError,
};

pub const RETRY_DURATION: Duration = Duration::from_secs(1);
pub const NMT_SWITCH_TIMEOUT: Duration = Duration::from_secs(1);
pub const NMT_SWITCH_ATTEMPTS: usize = 10;

/// Parametrize & Set up PDO mapping for cia402 compliant motor at given node_id
pub async fn motor_startup_task(
    node_id: u8,
    nmt_tx: mpsc::Sender<NmtState>,
    sdo: Arc<Mutex<SdoClient>>,
    parameters: &[SdoAction<'_>],
    rpdo_mapping: &'static [PdoMapping],
    tpdo_mapping: &'static [PdoMapping],
    mut event_rx: broadcast::Receiver<MotorEvent>,
) -> Result<(), DriveError> {
    trace!("Starting up motor at node id {node_id}");

    // Put the drive in NMT PreOperational, required for parametrisation & pdo mapping
    let state = NmtState::PreOperational;
    let mut attempt = 0;
    let mut nmt_event_rx = event_rx.resubscribe();

    loop {
        // Put the device in PreOperational
        nmt_tx
            .send(state.clone())
            .await
            .map_err(|err| DriveError::NMTSendError(state.clone(), err))?;

        // Wait for event indicating correct NMT state
        match timeout(NMT_SWITCH_TIMEOUT, nmt_event_rx.recv()).await {
            Ok(Ok(MotorEvent::NmtStateUpdate(new_state))) => {
                error!("new_state: {new_state:?}");
                // Got an event within the timeout
                if new_state == state {
                    break;
                }
            }
            Ok(Err(err)) => {
                // The channel closed before we got an event
                error!("Startup NMT PRE-OP: {err}");
            }
            Err(_) => {
                // Timeout expired
                warn!("Startup NMT PRE-OP: Timed out waiting for event");
            }
            Ok(Ok(event)) => {
                // Non-NMT event
                error!("Non NMT event : {event:?}");
            }
        }

        attempt += 1;
        if attempt >= NMT_SWITCH_ATTEMPTS {
            panic!(
                "Failed to switch device into NMT {state:?} after {NMT_SWITCH_ATTEMPTS} attempts, aborting"
            );
        }
    }

    // Parametrise this motor
    loop {
        trace!("Attempting to parametrise motor at node id {node_id}");
        if let Err(err) = parametrise_motor(node_id, parameters, sdo.clone()).await {
            warn!(
                "Parametrisation failed of motor at node id {node_id}: {err}, retrying in {}s",
                RETRY_DURATION.as_secs()
            );
            sleep(RETRY_DURATION).await;
        } else {
            info!("Succesful parametrisation of motor {node_id}");
            break;
        }
    }

    // Configure RPDO mapping
    trace!("Configuring RPDO_mapping of motor at node id {node_id}");
    loop {
        if let Err(err) = configure_pdo_mappings(node_id, sdo.clone(), rpdo_mapping).await {
            warn!(
                "RPDO mapping configuration failed of motor at node id {node_id}: {err}, retrying in {}s",
                RETRY_DURATION.as_secs()
            );
            sleep(RETRY_DURATION).await;
        } else {
            warn!("Succesful RPDO mapping for motor {node_id}");
            break;
        }
    }

    // Configure TPDO mapping
    trace!("Configuring TPDO_mapping of motor at node id {node_id}");
    loop {
        if let Err(err) = configure_pdo_mappings(node_id, sdo.clone(), tpdo_mapping).await {
            warn!(
                "TPDO mapping configuration failed of motor at node id {node_id}: {err}, retrying in {}s",
                RETRY_DURATION.as_secs()
            );
            sleep(RETRY_DURATION).await;
        } else {
            warn!("Succesful TPDO mapping for motor {node_id}");
            break;
        }
    }

    // Put the drive in NMT Operational
    let state = NmtState::Operational;
    let mut attempt = 0;
    let mut nmt_event_rx = event_rx.resubscribe();
    loop {
        // Put the device in Opertional
        nmt_tx
            .send(state.clone())
            .await
            .map_err(|err| DriveError::NMTSendError(state.clone(), err))?;

        // Wait for event indicating correct NMT state
        match timeout(NMT_SWITCH_TIMEOUT, nmt_event_rx.recv()).await {
            Ok(Ok(MotorEvent::NmtStateUpdate(new_state))) => {
                // Got an event within the timeout
                if new_state == state {
                    break;
                }
            }
            Ok(Err(err)) => {
                // The channel closed before we got an event
                error!("Startup NMT PRE-OP: {err}");
            }
            Err(_) => {
                // Timeout expired
                warn!("Startup NMT PRE-OP: Timed out waiting for event");
            }
            _ => {
                // Non-NMT event
            }
        }

        attempt += 1;
        if attempt >= NMT_SWITCH_ATTEMPTS {
            panic!(
                "Failed to switch device into NMT {state:?} after {NMT_SWITCH_ATTEMPTS} attempts, aborting"
            );
        }
    }

    Ok(())
}

pub async fn switch_to_nmt(
    state: NmtState,
    node_id: u8,
    nmt_tx: &mpsc::Sender<NmtState>,
    event_rx: broadcast::Receiver<MotorEvent>,
) -> Result<(), DriveError> {
    trace!("Switching motor {node_id} to {:?}", state.clone());
    nmt_tx
        .send(state.clone())
        .await
        .map_err(|err| DriveError::NMTSendError(state.clone(), err))?;

    wait_for_event(
        event_rx,
        MotorEvent::NmtStateUpdate(state.clone()),
        NMT_SWITCH_TIMEOUT,
    )
    .await
}

pub mod home;
pub mod parametrise;
pub mod params;
pub mod pdo_mapping;

use std::{sync::Arc, time::Duration};

use oze_canopen::sdo_client::SdoClient;
use tokio::{
    sync::{Mutex, broadcast, mpsc},
    time::sleep,
};
use tracing::*;

use crate::{
    comms::{pdo::mapping::PDOSet, sdo::SdoAction},
    driver::{
        event::MotorEvent,
        nmt::{NmtState, set_to_nmt_state},
        startup::{parametrise::parametrise_motor, pdo_mapping::configure_pdo_mappings},
    },
    error::{DriveError, InitialisationError},
};

pub const RETRY_DURATION: Duration = Duration::from_secs(1);

/// Parametrize & Set up PDO mapping for cia402 compliant motor at given node_id
pub async fn motor_startup_task(
    node_id: u8,
    nmt_tx: mpsc::Sender<NmtState>,
    sdo: Arc<Mutex<SdoClient>>,
    parameters: &[SdoAction<'_>],
    default_pdo_set: &'static PDOSet,
    event_rx: broadcast::Receiver<MotorEvent>,
) -> Result<(), InitialisationError> {
    trace!("Starting up motor at node id {node_id}");

    // Put the drive in NMT PreOperational, required for parametrisation & pdo mapping
    set_to_nmt_state(NmtState::PreOperational, &nmt_tx, event_rx.resubscribe())
        .await
        .map_err(|_| InitialisationError::ParametrisationNMTPreOp(node_id))?;

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
    trace!("Configuring default RPDO_mapping of motor at node id {node_id}");
    loop {
        if let Err(err) = configure_pdo_mappings(node_id, sdo.clone(), default_pdo_set.rpdos).await
        {
            warn!(
                "RPDO mapping configuration failed of motor at node id {node_id}: {err}, retrying in {}s",
                RETRY_DURATION.as_secs()
            );
            sleep(RETRY_DURATION).await;
        } else {
            info!("Succesful RPDO mapping for motor {node_id}");
            break;
        }
    }

    // Configure TPDO mapping
    trace!("Configuring default TPDO_mapping of motor at node id {node_id}");
    loop {
        if let Err(err) = configure_pdo_mappings(node_id, sdo.clone(), default_pdo_set.tpdos).await
        {
            warn!(
                "TPDO mapping configuration failed of motor at node id {node_id}: {err}, retrying in {}s",
                RETRY_DURATION.as_secs()
            );
            sleep(RETRY_DURATION).await;
        } else {
            info!("Succesful TPDO mapping for motor {node_id}");
            break;
        }
    }

    // Put the drive in NMT Operational
    set_to_nmt_state(NmtState::Operational, &nmt_tx, event_rx.resubscribe())
        .await
        .map_err(|_| InitialisationError::ParametrisationNMTOp(node_id))?;

    trace!("Device reporst NMT Opertional -> Startup Completed!");

    Ok(())
}

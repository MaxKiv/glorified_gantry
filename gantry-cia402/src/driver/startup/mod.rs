pub mod parametrise;
pub mod params;
pub mod pdo_mapping;

use std::{sync::Arc, time::Duration};

use oze_canopen::sdo_client::SdoClient;
use tokio::{
    sync::{Mutex, broadcast, oneshot},
    time::sleep,
};
use tracing::*;

use crate::{
    comms::{pdo::mapping::PdoMapping, sdo::SdoAction},
    driver::{
        event::MotorEvent,
        startup::{parametrise::parametrise_motor, pdo_mapping::configure_pdo_mappings},
    },
    error::DriveError,
};

pub const PARAMETRISATION_RETRY_DURATION: Duration = Duration::from_secs(1);

/// Parametrize & Set up PDO mapping for cia402 compliant motor at given node_id
pub async fn motor_startup_task(
    node_id: u8,
    sdo: Arc<Mutex<SdoClient>>,
    parameters: &[SdoAction<'_>],
    rpdo_mapping: &'static [PdoMapping],
    tpdo_mapping: &'static [PdoMapping],
    event_tx: broadcast::Sender<MotorEvent>,
    startup_completed_tx: oneshot::Sender<bool>,
) -> Result<(), DriveError> {
    trace!("Starting up motor at node id {node_id}");
    loop {
        trace!("Attempting to parametrise motor at node id {node_id}");
        if let Err(err) = parametrise_motor(node_id, parameters, sdo.clone()).await {
            warn!("Failed to parametrise motor for {node_id}: {err}");
            if let Err(err) = event_tx.send(MotorEvent::Fault {
                code: 0,
                description: format!("Failed to parametrise motor at node id {node_id}: {err}")
                    .to_string(),
            }) {
                error!("Failed to send Event: {err} - We are on our own",);
            }
        } else {
            warn!(
                "Parametrisation failed of motor at node id {node_id}, retrying in {}s",
                PARAMETRISATION_RETRY_DURATION.as_secs()
            );
            sleep(PARAMETRISATION_RETRY_DURATION).await;
            break;
        }
    }

    trace!("Configuring RPDO_mapping of motor at node id {node_id}");
    loop {
        if let Err(err) = configure_pdo_mappings(node_id, sdo.clone(), rpdo_mapping).await {
            warn!("Failed to configure_pdo_mappings for {node_id}: {err}");
            if let Err(err) = event_tx.send(MotorEvent::Fault {
                code: 0,
                description: format!(
                    "Failed to configure RPDO mapping for motor at node id {node_id}"
                )
                .to_string(),
            }) {
                error!("Failed to send Event: {err} - We are on our own",);
            }
        } else {
            warn!(
                "RPDO mapping configuration failed of motor at node id {node_id}, retrying in {}s",
                PARAMETRISATION_RETRY_DURATION.as_secs()
            );
            sleep(PARAMETRISATION_RETRY_DURATION).await;
            break;
        }
    }

    trace!("Configuring PPDO_mapping of motor at node id {node_id}");
    loop {
        if let Err(err) = configure_pdo_mappings(node_id, sdo.clone(), tpdo_mapping).await {
            warn!("Failed to configure_pdo_mappings for {node_id}: {err}");
            if let Err(err) = event_tx.send(MotorEvent::Fault {
                code: 0,
                description: format!(
                    "Failed to configure TPDO mapping for motor at node id {node_id}"
                )
                .to_string(),
            }) {
                error!("Failed to send Event: {err} - We are on our own",);
            }
        } else {
            warn!(
                "TPDO mapping configuration failed of motor at node id {node_id}, retrying in {}s",
                PARAMETRISATION_RETRY_DURATION.as_secs()
            );
            sleep(PARAMETRISATION_RETRY_DURATION).await;
            break;
        }
    }

    startup_completed_tx.send(true);
    Ok(())
}

pub mod home;
pub mod parametrise;
pub mod params;
pub mod pdo_mapping;

use std::{sync::Arc, time::Duration};

use oze_canopen::sdo_client::SdoClient;
use tokio::{
    sync::{Mutex, broadcast, mpsc, oneshot},
    time::sleep,
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

    switch_to_nmt(
        NmtState::PreOperational,
        node_id,
        nmt_tx.clone(),
        &mut event_rx,
    )
    .await?;

    loop {
        trace!("Attempting to parametrise motor at node id {node_id}");
        if let Err(err) = parametrise_motor(node_id, parameters, sdo.clone()).await {
            warn!("Failed to parametrise motor for {node_id}: {err}");
        } else {
            warn!(
                "Parametrisation failed of motor at node id {node_id}, retrying in {}s",
                RETRY_DURATION.as_secs()
            );
            sleep(RETRY_DURATION).await;
            break;
        }
    }

    trace!("Configuring RPDO_mapping of motor at node id {node_id}");
    loop {
        if let Err(err) = configure_pdo_mappings(node_id, sdo.clone(), rpdo_mapping).await {
            warn!("Failed to configure_pdo_mappings for {node_id}: {err}");
        } else {
            warn!(
                "RPDO mapping configuration failed of motor at node id {node_id}, retrying in {}s",
                RETRY_DURATION.as_secs()
            );
            sleep(RETRY_DURATION).await;
            break;
        }
    }

    trace!("Configuring PPDO_mapping of motor at node id {node_id}");
    loop {
        if let Err(err) = configure_pdo_mappings(node_id, sdo.clone(), tpdo_mapping).await {
            warn!("Failed to configure_pdo_mappings for {node_id}: {err}");
        } else {
            warn!(
                "TPDO mapping configuration failed of motor at node id {node_id}, retrying in {}s",
                RETRY_DURATION.as_secs()
            );
            sleep(RETRY_DURATION).await;
            break;
        }
    }

    switch_to_nmt(NmtState::Operational, node_id, nmt_tx, &mut event_rx).await?;

    Ok(())
}

pub async fn switch_to_nmt(
    state: NmtState,
    node_id: u8,
    nmt_tx: mpsc::Sender<NmtState>,
    event_rx: &mut broadcast::Receiver<MotorEvent>,
) -> Result<(), DriveError> {
    loop {
        trace!("Switching motor {node_id} to {:?}", state.clone());
        nmt_tx
            .send(state.clone())
            .await
            .map_err(|err| DriveError::NMTSendError(state.clone(), err))?;
        if let Ok(_) = wait_for_event(
            event_rx,
            MotorEvent::NmtStateUpdate(state.clone()),
            NMT_SWITCH_TIMEOUT,
        )
        .await
        {
            trace!("Motor {node_id} is in {state:?}");
            return Ok(());
        }
    }
}

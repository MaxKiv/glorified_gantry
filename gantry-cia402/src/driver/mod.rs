pub mod builder;
pub mod command;
pub mod event;
pub mod nmt;
pub mod oms;
pub mod receiver;
pub mod startup;
pub mod state;
pub mod update;

use std::sync::Arc;

use crate::{
    comms::{
        pdo::{Pdo, mapping::PdoMapping},
        sdo::SdoAction,
    },
    driver::{
        command::MotorCommand,
        event::MotorEvent,
        nmt::{NmtState, nmt_task},
        receiver::{setpoint_manager::SetpointManager, subscriber::handle_feedback},
        startup::motor_startup_task,
        state::{orchestrator::cia402_orchestrator_task, state_machine::cia402_state_machine_task},
        update::publisher::publish_updates,
    },
    error::DriveError,
    log::log_events,
};

use anyhow::Result;
use oze_canopen::{interface::CanOpenInterface, sdo_client::SdoClient};
use tokio::{
    sync::{Mutex, broadcast, mpsc},
    task::{self, JoinHandle},
};
use tracing::*;

/// CiA-402 driver built on top of a CANopen protocol manager
pub struct Cia402Driver {
    pub node_id: u8,
    pub cmd_tx: broadcast::Sender<MotorCommand>,
    pub nmt_tx: mpsc::Sender<NmtState>,
    pub event_rx: broadcast::Receiver<MotorEvent>,
    canopen: CanOpenInterface,
    _handles: Vec<JoinHandle<()>>,
    sdo: Arc<Mutex<SdoClient>>,
}

impl Cia402Driver {
    /// Initialize a new Cia402Driver to manage all CiA-402 related interactions with a single motor
    /// connected to the given CANopen interface on the given node id.
    /// It requires motor parametrisation defined as a slice of SdoActions, and a valid TPDO and
    /// RPDO mapping for this motor.
    /// When calling this a few different tokio::tasks are spawned, each responsible for different
    /// parts of the cia402 specification.
    /// Dropping this also cancels the managed tasks.
    /// NOTE: the initialisation order matters here, you could use the typestate pattern to encode
    /// that information in the type system, but who has the time?
    async fn spawn_tasks(
        node_id: u8,
        canopen: CanOpenInterface,
        parameters: &'static [SdoAction<'_>],
        rpdo_mapping_set: &'static [PdoMapping],
        tpdo_mapping_set: &'static [PdoMapping],
    ) -> Result<Self, DriveError> {
        // Track task handles that we are about to spawn to bind their lifetimes to this object
        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        // Initialize input interfaces
        let (cmd_tx, cmd_rx) = tokio::sync::broadcast::channel(10);

        // Initialize output interfaces
        let (event_tx, event_rx): (
            broadcast::Sender<MotorEvent>,
            broadcast::Receiver<MotorEvent>,
        ) = tokio::sync::broadcast::channel(10);

        // Early resubscribe the event receivers so components do not miss anything that happend
        // before they spawned
        let event_rx_logger = event_rx.resubscribe();
        let event_rx_nmt = event_rx.resubscribe();
        let event_rx_startup = event_rx.resubscribe();
        let event_rx_cia402 = event_rx.resubscribe();
        let event_rx_setpoint_manager = event_rx.resubscribe();
        let event_tx_feedback = event_tx.clone();
        let event_tx_cia402_sm = event_tx.clone();

        let cmd_rx_cia402_orch = cmd_rx.resubscribe();
        let cmd_rx_publisher = cmd_rx.resubscribe();

        let canopen_feedback = canopen.clone();
        let canopen_nmt = canopen.clone();

        // Initialize the event_logger
        trace!("Starting Event Logger for node id {node_id}");
        handles.push(spawn_logged("EVENT", async move {
            log_events(event_rx_logger, node_id).await
        }));

        // Start the device feedback task responsible for receiving and parsing device feedback,
        // and broadcasting these as events
        trace!("Starting device feedback handler for motor with node id {node_id}");
        handles.push(spawn_logged("FEEDBACK", async move {
            handle_feedback(
                node_id,
                canopen_feedback,
                tpdo_mapping_set,
                event_tx_feedback,
            )
            .await
        }));

        // Initialize the Cia402 Orchestrator -> State Machine command channel
        let (sm_cmd_tx, sm_cmd_rx) = tokio::sync::mpsc::channel(10);
        // Initialize the State machine -> Orchestrator state feedback channel
        let (sm_state_tx, sm_state_rx) = tokio::sync::broadcast::channel(10);

        // Initialize Cia402 Task -> Publisher channel
        let (state_update_tx, state_update_rx) = tokio::sync::mpsc::channel(10);

        // Initialize the NMT Task channel
        let (nmt_tx, nmt_rx) = tokio::sync::mpsc::channel(10);

        // Get the SDO client for this node id, we use this to make SDO read/writes
        let sdo = canopen
            .clone()
            .get_sdo_client(node_id)
            .expect("Unable to construct SDO client for node id {node_id}");

        // Get the PDO client for this node id, we use this to manage R/TPDOs
        trace!("Starting PDO task for device {node_id}");
        let (pdo_handle, pdo_tx) = Pdo::init(canopen.clone(), node_id, rpdo_mapping_set)
            .expect("unable to construct PDO client for node id {node_id}");
        handles.push(pdo_handle);

        // Start the setpoint manager for this node, this encapsulates reactive setpoint logic by clearing CW bit 4 when device posts SW 12
        let (setpoint_manager_handle, new_setpoint_tx) =
            SetpointManager::init(event_rx_setpoint_manager, pdo_tx.clone());
        handles.push(setpoint_manager_handle);

        // Start the NMT task
        trace!("Starting NMT State Machine task for motor with node id {node_id}");
        handles.push(spawn_logged("NMT", async move {
            nmt_task(node_id, canopen_nmt, nmt_rx, event_rx_nmt).await
        }));

        // Start the cia402 state machine task, this is responsible for
        // tracking the motors current cia402 state and single transition
        trace!("Starting Cia402 State Machine for motor with node id {node_id}");
        handles.push(spawn_logged("CIA-SM", async move {
            cia402_state_machine_task(
                event_rx_cia402,
                state_update_tx,
                sm_state_tx,
                sm_cmd_rx,
                event_tx_cia402_sm,
            )
            .await
        }));

        trace!("Starting Cia402 Orchestrator for motor with node id {node_id}");
        handles.push(spawn_logged("CIA-OR", async move {
            cia402_orchestrator_task(sm_cmd_tx, sm_state_rx, cmd_rx_cia402_orch).await
        }));

        // Start the publisher task, responsible for update aggregation and device communication
        trace!("Starting update publisher task for motor with node id {node_id}");
        handles.push(spawn_logged("UPDATE", async move {
            publish_updates(
                pdo_tx.clone(),
                state_update_rx,
                cmd_rx_publisher,
                new_setpoint_tx,
            )
            .await
        }));

        // Start the startup task for this motor, this does parametrisation and configures pdo mapping
        trace!("Performing Startup for motor at node id {node_id}");
        if let Err(err) = motor_startup_task(
            node_id,
            nmt_tx.clone(),
            sdo.clone(),
            parameters,
            rpdo_mapping_set,
            tpdo_mapping_set,
            event_rx_startup,
        )
        .await
        {
            error!("Unable to perform startup for motor at node id {node_id}: {err}");
            return Err(err);
        }
        trace!("Startup done for motor at node id {node_id}");

        // Drive is now parametrised, T/RPDO are configured and in NMT::Operational
        info!("Cia402Driver for node id {node_id} constructed and initialized");
        Ok(Cia402Driver {
            node_id,
            cmd_tx,
            nmt_tx,
            event_rx: event_rx.resubscribe(),
            canopen,
            _handles: handles,
            sdo,
        })
    }

    pub async fn shutdown(self) {
        let _ = self.cmd_tx.send(MotorCommand::Halt);
        let _ = self.cmd_tx.send(MotorCommand::Disable);

        info!("Shutting down node {}", self.node_id);
        for handle in self._handles {
            handle.abort();
        }
    }
}

/// Helper that spawns a task and logs error if it ever exits
fn spawn_logged<F>(name: &'static str, fut: F) -> JoinHandle<()>
where
    F: std::future::Future<Output = Result<(), DriveError>> + Send + 'static,
{
    tokio::spawn(async move {
        if let Err(e) = fut.await {
            error!("{name} task failed: {e:?}");
        }
    })
}

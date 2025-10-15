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
        receiver::subscriber::handle_feedback,
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
    pub async fn init(
        node_id: u8,
        canopen: CanOpenInterface,
        parameters: &'static [SdoAction<'_>],
        rpdo_mapping_set: &'static [PdoMapping],
        tpdo_mapping_set: &'static [PdoMapping],
    ) -> Result<Self, DriveError> {
        // Track task handles that we are about to spawn
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
        let event_tx_feedback = event_tx.clone();
        let event_tx_cia402_sm = event_tx.clone();

        let cmd_rx_cia402_orch = cmd_rx.resubscribe();
        let cmd_rx_publisher = cmd_rx.resubscribe();

        let canopen_feedback = canopen.clone();
        let canopen_nmt = canopen.clone();
        // let canopen_logger = canopen.clone();

        // Initialize the event_logger
        handles.push(task::spawn(async move {
            match log_events(event_rx_logger, node_id).await {
                Ok(_) => error!("Event logger finished succesfully, this should never happen"),
                Err(err) => error!("Event Logger panicked: {err}"),
            }
        }));

        // trace!("Starting canopen pretty logger");
        // handles.push(tokio::task::spawn(async move {
        //     match crate::log::log_canopen_pretty(canopen_logger).await {
        //         Ok(_) => {
        //             error!("Update Publisher task finished succesfully, this should never happen")
        //         }
        //         Err(err) => {
        //             error!("Update Publisher task panicked: {err}")
        //         }
        //     }
        // }));

        // Start the device feedback task responsible for receiving and parsing device feedback,
        // and broadcasting these as events
        trace!("Starting device feedback handler for motor with node id {node_id}");
        handles.push(task::spawn(async move {
            handle_feedback(
                node_id,
                canopen_feedback,
                tpdo_mapping_set,
                event_tx_feedback,
            )
            .await;
            error!("Feedback task finished succesfully, this should never happen");
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
        let pdo = Pdo::new(canopen.clone(), node_id, rpdo_mapping_set)
            .expect("unable to construct PDO client for node id {node_id}");

        // Start the NMT task
        trace!("Starting NMT State Machine task for motor with node id {node_id}");
        handles.push(task::spawn(async move {
            nmt_task(node_id, canopen_nmt, nmt_rx, event_rx_nmt).await;
            error!("NMT task finished succesfully, this should never happen");
        }));

        // Start the cia402 state machine task, this is responsible for
        // tracking the motors current cia402 state and single transition
        trace!("Starting Cia402 State Machine for motor with node id {node_id}");
        handles.push(task::spawn(async move {
            cia402_state_machine_task(
                event_rx_cia402,
                state_update_tx,
                sm_state_tx,
                sm_cmd_rx,
                event_tx_cia402_sm,
            )
            .await;
            error!("Cia402 State machine task finished succesfully, this should never happen");
        }));

        trace!("Starting Cia402 Orchestrator for motor with node id {node_id}");
        handles.push(task::spawn(async move {
            cia402_orchestrator_task(sm_cmd_tx, sm_state_rx, cmd_rx_cia402_orch).await;
            error!("Cia402 Orchestrator task finished succesfully, this should never happen");
        }));

        // Start the publisher task, responsible for update aggregation and device communication
        trace!("Starting update publisher task for motor with node id {node_id}");
        handles.push(tokio::task::spawn(async move {
            publish_updates(pdo, state_update_rx, cmd_rx_publisher).await;
            error!("Update Publisher task finished succesfully, this should never happen");
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

        // let mut rx_test = canopen.clone().rx;
        // match tokio::time::timeout(Duration::from_secs(1), rx_test.recv()).await {
        //     Ok(Ok(frame)) => error!("yay received frame: {frame:?}"),
        //     Ok(Err(err)) => error!("no timeout, but error: {err}"),
        //     Err(err) => error!("timeout elapsed - unable to receive after startup: {err}"),
        // }

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
}

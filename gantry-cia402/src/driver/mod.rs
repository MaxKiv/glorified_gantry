pub mod command;
pub mod event;
pub mod nmt;
pub mod oms;
pub mod receiver;
pub mod startup;
pub mod state;
pub mod update;

use crate::{
    comms::{
        pdo::{Pdo, mapping::PdoMapping},
        sdo::SdoAction,
    },
    driver::{
        command::MotorCommand, event::MotorEvent, nmt::nmt_task,
        receiver::subscriber::handle_feedback, startup::motor_startup_task, state::cia402_task,
        update::publisher::publish_updates,
    },
    error::DriveError,
};

use anyhow::Result;
use oze_canopen::interface::CanOpenInterface;
use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task::{self, JoinHandle},
};
use tracing::*;

/// CiA-402 driver built on top of a CANopen protocol manager
pub struct Cia402Driver {
    pub node_id: u8,
    cmd_tx: broadcast::Sender<MotorCommand>,
    pub event_rx: broadcast::Receiver<MotorEvent>,
    handles: Vec<JoinHandle<()>>,
}

impl Cia402Driver {
    /// Initialize a new Cia402Driver to manage all CiA-402 related interactions with a single motor
    /// connected to the given CANopen interface on the given node id.
    /// It requires motor parametrisation defined as a slice of SdoActions, and a valid TPDO and
    /// RPDO mapping for this motor
    /// When calling this a few different tokio::tasks are spawned, each responsible for different
    /// parts of the cia402 specification
    /// Dropping this also cancels the managed tasks
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

        // Initialize Cia402 Task -> Publisher channel
        let (state_update_tx, state_update_rx) = tokio::sync::mpsc::channel(10);

        // Initialize the NMT Task channel
        let (nmt_tx, nmt_rx) = tokio::sync::mpsc::channel(10);

        // Get the SDO client for this node id, we use this to make SDO read/writes
        let sdo = canopen
            .get_sdo_client(node_id)
            .unwrap_or_else(|| panic!("Unable to construct SDO client for node id {node_id}"));

        // Get the PDO client for this node id, we use this to manage R/TPDOs
        let pdo = Pdo::new(canopen.clone(), node_id, rpdo_mapping_set)
            .unwrap_or_else(|_| panic!("unable to construct PDO client for node id {node_id}"));

        // Start the NMT task
        trace!("Starting NMT State Machine task for motor with node id {node_id}");
        handles.push(task::spawn(nmt_task(
            node_id,
            canopen.clone(),
            nmt_rx,
            event_rx.resubscribe(),
        )));

        // Start the startup task for this motor, this does parametrisation and configures pdo mapping
        trace!("Starting Startup Task for motor with node id {node_id}");
        motor_startup_task(
            node_id,
            nmt_tx.clone(),
            sdo,
            parameters,
            rpdo_mapping_set,
            tpdo_mapping_set,
            event_rx.resubscribe(),
        )
        .await?;

        // Start the cia402 state machine task, this is responsible for managing cia402 state
        // transitions
        trace!("Starting Cia402 State Machine task for motor with node id {node_id}");
        task::spawn(cia402_task(
            event_rx.resubscribe(),
            cmd_rx.resubscribe(),
            state_update_tx,
        ));

        // Start the publisher task, responsible for update aggregation and device communication
        trace!("Starting update publisher task for motor with node id {node_id}");
        handles.push(tokio::task::spawn(publish_updates(
            pdo,
            state_update_rx,
            cmd_rx.resubscribe(),
        )));

        // Start the device feedback task responsible for receiving and parsing device feedback,
        // and broadcasting these as events
        trace!("Starting device feedback handler for motor with node id {node_id}");
        handles.push(task::spawn(handle_feedback(
            node_id,
            canopen.clone(),
            tpdo_mapping_set,
            event_tx,
        )));

        Ok(Cia402Driver {
            node_id,
            cmd_tx,
            event_rx,
            handles,
        })
    }
}

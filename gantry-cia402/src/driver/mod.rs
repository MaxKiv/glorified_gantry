pub mod builder;
pub mod command;
pub mod cyclic;
pub mod event;
pub mod identifier;
pub mod nmt;
pub mod oms;
pub mod receiver;
pub mod startup;
pub mod state;
pub mod update;

use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use crate::{
    comms::{
        pdo::{Pdo, mapping::PDOSet},
        sdo::SdoAction,
    },
    driver::{
        command::MotorCommand,
        event::MotorEvent,
        identifier::Cia402Identifier,
        nmt::{NmtState, nmt_task},
        receiver::{setpoint_manager::SetpointManager, subscriber::handle_feedback},
        startup::motor_startup_task,
        state::{orchestrator::cia402_orchestrator_task, state_machine::cia402_state_machine_task},
        update::publisher::publish_updates,
    },
    error::{DriveError, InitialisationError},
    log::log_events,
};

use anyhow::Result;
use oze_canopen::{interface::CanOpenInterface, sdo_client::SdoClient};
use tokio::{
    sync::{Mutex, broadcast, mpsc},
    task::{AbortHandle, JoinHandle, JoinSet},
    time::Instant,
};
use tracing::*;

/// Cia402Driver configured standalone, default
pub struct Standalone;
/// Cia402Driver configured as axis master
pub struct AxisMaster;
/// Cia402Driver configured as axis slave
pub struct AxisSlave;

/// CiA-402 driver built on top of a CANopen protocol manager
pub struct Cia402Driver<Mode = Standalone> {
    pub identifier: Cia402Identifier,
    pub cmd_tx: broadcast::Sender<MotorCommand>,
    pub cmd_rx: broadcast::Receiver<MotorCommand>,
    pub nmt_tx: mpsc::Sender<NmtState>,
    pub event_rx: broadcast::Receiver<MotorEvent>,
    canopen: CanOpenInterface,
    joinset: JoinSet<()>,
    sdo: Arc<Mutex<SdoClient>>,
    _mode: std::marker::PhantomData<Mode>,
}

impl<Mode> Drop for Cia402Driver<Mode> {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(MotorCommand::Disable);

        // TMTM-40: Figure out what a safe state is, and make sure we go there on [`Drop`] before comms tasks are aborted
        // For instance use an atomic bool or oneshot channel to communicate between drop and Comms task:
        // while !self.shutdown_ack.as_ref().unwrap().load(Ordering::SeqCst) {
        //     // Small yield to avoid busy-wait
        //     std::thread::sleep(Duration::from_micros(10));
        // }

        self.joinset.abort_all();
    }
}

impl<Mode> Cia402Driver<Mode> {
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
        identifier: Cia402Identifier,
        canopen: CanOpenInterface,
        parameters: &'static [SdoAction<'_>],
        default_pdo_set: &'static PDOSet,
        minimal_pdo_set: &'static PDOSet,
        sync_rx: broadcast::Receiver<Instant>,
        cmd_tx: broadcast::Sender<MotorCommand>,
        cmd_rx: broadcast::Receiver<MotorCommand>,
    ) -> Result<Self, InitialisationError> {
        // Track task handles that we are about to spawn to bind their lifetimes to this object
        let mut handles = JoinSet::new();
        let node_id = identifier.node_id;

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
        let event_rx_updater = event_rx.resubscribe();
        let event_tx_feedback = event_tx.clone();
        let event_tx_cia402_sm = event_tx.clone();

        let cmd_rx_cia402_orch = cmd_rx.resubscribe();
        let cmd_rx_publisher = cmd_rx.resubscribe();

        let canopen_feedback = canopen.clone();
        let canopen_nmt = canopen.clone();

        // Initialize the event_logger
        trace!("Starting Event Logger for motor: {}", identifier);
        spawn_logged_joinset(&mut handles, "EVENT", async move {
            log_events(event_rx_logger, node_id).await
        });

        // Start the device feedback task responsible for receiving and parsing device feedback,
        // and broadcasting these as events
        trace!("Starting device feedback handler for motor {identifier}");
        spawn_logged_joinset(&mut handles, "FEEDBACK", async move {
            handle_feedback(
                node_id,
                canopen_feedback,
                default_pdo_set.tpdos,
                event_tx_feedback,
            )
            .await
        });

        // Initialize the Cia402 Orchestrator -> State Machine command channel
        let (sm_cmd_tx, sm_cmd_rx) = tokio::sync::mpsc::channel(10);
        // Initialize the State machine -> Orchestrator state feedback channel
        let (sm_state_tx, sm_state_rx) = tokio::sync::broadcast::channel(10);

        // Initialize Cia402 Task -> Publisher channel
        let (state_update_tx, state_update_rx) = tokio::sync::mpsc::channel(10);

        // Initialize the NMT Task channel
        let (nmt_tx, nmt_rx) = tokio::sync::mpsc::channel(10);

        // Start the NMT task
        trace!("Starting NMT State Machine task for motor {identifier}");
        spawn_logged_joinset(&mut handles, "NMT", async move {
            nmt_task(node_id, canopen_nmt, nmt_rx, event_rx_nmt).await
        });

        // Get the SDO client for this node id, we use this to make SDO read/writes
        let Some(sdo) = canopen.clone().get_sdo_client(node_id) else {
            return Err(InitialisationError::SdoClientConstructionFailed(identifier));
        };

        // Get the PDO client for this node id, we use this to manage R/TPDOs
        trace!("Starting PDO task for motor {identifier}");
        let (_pdo_handle, pdo_tx) = Pdo::init(
            canopen.clone(),
            node_id,
            default_pdo_set,
            minimal_pdo_set,
            &mut handles,
        )?;

        // Start the setpoint manager for this device, handles setpoint writes and OMS specifics
        // like profile position handshaking
        let (_setpoint_manager_handle, new_setpoint_tx, cs_mode_tx) = SetpointManager::init(
            node_id,
            event_rx_setpoint_manager,
            pdo_tx.clone(),
            sync_rx,
            &mut handles,
        );

        // Start the cia402 state machine task, this is responsible for
        // tracking the motors current cia402 state and single transition
        trace!("Starting Cia402 State Machine for motor {identifier}");
        spawn_logged_joinset(&mut handles, "CIA-SM", async move {
            cia402_state_machine_task(
                event_rx_cia402,
                state_update_tx,
                sm_state_tx,
                sm_cmd_rx,
                event_tx_cia402_sm,
            )
            .await
        });

        trace!("Starting Cia402 Orchestrator for motor {identifier}");
        spawn_logged_joinset(&mut handles, "CIA-OR", async move {
            cia402_orchestrator_task(sm_cmd_tx, sm_state_rx, cmd_rx_cia402_orch).await
        });

        // Start the publisher task, responsible for update aggregation and device communication
        trace!("Starting update publisher task for motor {identifier}");
        let nmt_tx_updater = nmt_tx.clone();
        let sdo_updater = sdo.clone();
        spawn_logged_joinset(&mut handles, "UPDATE", async move {
            publish_updates(
                pdo_tx.clone(),
                state_update_rx,
                cmd_rx_publisher,
                new_setpoint_tx,
                cs_mode_tx,
                nmt_tx_updater,
                event_rx_updater,
                sdo_updater,
                node_id,
            )
            .await
        });

        // Start the startup task for this motor, this does parametrisation and configures pdo mapping
        trace!("Performing Startup for motor {identifier}");
        if let Err(err) = motor_startup_task(
            identifier.clone(),
            nmt_tx.clone(),
            sdo.clone(),
            parameters,
            default_pdo_set,
            event_rx_startup,
        )
        .await
        {
            error!("Startup error: {err}, releasing resources and exiting.");
            handles.shutdown().await;
            return Err(err);
        }
        trace!("Startup done for motor {identifier}");

        // Drive is now parametrised, T/RPDO are configured and in NMT::Operational
        info!("Cia402Driver for motor {identifier} constructed and initialized");
        Ok(Cia402Driver::<Mode> {
            identifier,
            cmd_tx,
            cmd_rx,
            nmt_tx,
            event_rx: event_rx.resubscribe(),
            canopen,
            joinset: handles,
            sdo,
            _mode: std::marker::PhantomData::<Mode>,
        })
    }

    pub fn get_cmd_tx_channel(&self) -> broadcast::Sender<MotorCommand> {
        self.cmd_tx.clone()
    }

    pub fn get_cmd_rx_channel(&self) -> broadcast::Receiver<MotorCommand> {
        self.cmd_rx.resubscribe()
    }
}

/// Helper that spawns a task and logs error if it ever exits
pub fn spawn_logged<F>(name: &'static str, fut: F) -> JoinHandle<()>
where
    F: std::future::Future<Output = Result<(), DriveError>> + Send + 'static,
{
    tokio::spawn(async move {
        if let Err(e) = fut.await {
            error!("{name} task failed: {e:?}");
        }
    })
}

/// Helper that spawns a task and logs error if it ever exits
pub fn spawn_logged_joinset<F>(set: &mut JoinSet<()>, name: &'static str, fut: F) -> AbortHandle
where
    F: std::future::Future<Output = Result<(), DriveError>> + Send + 'static,
{
    set.spawn(async move {
        if let Err(e) = fut.await {
            error!("{name} task failed: {e:?}");
        }
    })
}

pub mod command;
pub mod event;
pub mod feedback;
pub mod nmt;
pub mod oms;
pub mod router;
pub mod startup;
pub mod state;
pub mod update;

use crate::{
    comms::{
        pdo::{Pdo, mapping::PdoMapping},
        sdo::SdoAction,
    },
    driver::{
        command::MotorCommand,
        event::MotorEvent,
        nmt::Nmt,
        oms::OmsHandler,
        router::command_router,
        startup::motor_startup_task,
        state::{Cia402State, Cia402StateMachine},
        update::publisher::publish_updates,
    },
    error::DriveError,
    od::oms::{
        DEFAULT_POSITIONMODE_FLAGS, PositionModeFlags, PositionSetpoint, Setpoint, TorqueSetpoint,
        VelocitySetpoint,
    },
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
    cmd_tx: mpsc::Sender<MotorCommand>,
    event_rx: broadcast::Receiver<MotorEvent>,
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
        parameters: &[SdoAction<'_>],
        rpdo_mapping: &'static [PdoMapping],
        tpdo_mapping: &'static [PdoMapping],
    ) -> Result<Self, DriveError> {
        // Initialize input interfaces
        let (cmd_tx, cmd_rx): (mpsc::Sender<MotorCommand>, mpsc::Receiver<MotorCommand>) =
            tokio::sync::mpsc::channel(10);

        // Initialize output interfaces
        let (event_tx, event_rx): (
            broadcast::Sender<MotorEvent>,
            broadcast::Receiver<MotorEvent>,
        ) = tokio::sync::broadcast::channel(10);

        // Get the SDO client for this node id, we use this to make SDO read/writes
        let sdo = canopen.get_sdo_client(node_id).expect(format!(
            "Unable to construct SDO client for node id {node_id}"
        ));
        // Get the PDO client for this node id, we use this to manage R/TPDOs
        let pdo = Pdo::new(canopen, node_id, tpdo_mapping, rpdo_mapping);

        // Track task handles that we are about to spawn
        let handles = Vec::new();

        // Start the NMT task, this continously attempts to put this motor into NMT::Operational
        trace!("Starting NMT State Machine task for motor with node id {node_id}");
        let (nmt_feedback_tx, nmt_feedback_rx) = tokio::sync::mpsc::channel(10);
        handles.push(Nmt::init(node_id, canopen, nmt_feedback_rx));

        // Start the startup task for this motor, this does parametrisation and sets pdo mapping
        trace!("Starting Startup Task for motor with node id {node_id}");
        let (startup_completed_tx, startup_completed_rx): (
            oneshot::Sender<bool>,
            oneshot::Receiver<bool>,
        ) = tokio::sync::oneshot::channel();
        let startup = task::spawn(motor_startup_task(
            node_id,
            sdo,
            parameters,
            rpdo_mapping,
            tpdo_mapping,
            event_tx.clone(),
            startup_completed_tx,
        ));
        // Wait for the startup task to finish to make sure the motor is in a known state before
        // proceeding
        if let Err(err) = startup.await {
            error!("Startup task failed for motor with node id {node_id}: {err}");
        }

        // Start the cia402 state machine task, this is responsible for managing cia402 state
        // transitions
        trace!("Starting Cia402 State Machine task for motor with node id {node_id}");
        let (state_cmd_tx, state_cmd_rx) = tokio::sync::mpsc::channel(10);
        let (state_update_tx, state_update_rx) = tokio::sync::mpsc::channel(10);
        let (state_feedback_tx, state_feedback_rx) = tokio::sync::mpsc::channel(10);
        handles.push(Cia402StateMachine::init(
            node_id,
            canopen,
            state_cmd_rx,
            state_feedback_rx,
            state_update_tx,
        ));

        // Start the OMS task for this motor, this is responsible for translating pos/vel/torque
        // commands into their respective setpoints
        trace!("Starting Operational Mode Specific task for motor with node id {node_id}");
        let (setpoint_cmd_tx, setpoint_cmd_rx) = tokio::sync::mpsc::channel(10);
        let (setpoint_update_tx, setpoint_update_rx) = tokio::sync::mpsc::channel(10);
        handles.push(OmsHandler::init(
            node_id,
            canopen,
            setpoint_cmd_rx,
            setpoint_update_tx,
        ));

        // Start the command router which is responsible for routing the incoming commands to
        // either the cia402 state machine or the OMS handler
        trace!("Starting command router motor with node id {node_id}");
        handles.push(task::spawn(command_router(
            cmd_rx,
            state_cmd_tx,
            setpoint_cmd_tx,
            event_tx,
        )));

        // Start the update publisher which is responsible for aggregating the validated device
        // updates from the cia402 state machine and OMS handler, and translating those into the
        // correct OD updates for the device
        trace!("Starting update publisher task for motor with node id {node_id}");
        handles.push(tokio::task::spawn(publish_updates(
            pdo,
            state_update_rx,
            setpoint_update_rx,
        )));

        // Start teh device feedback task responsible for receiving and parsing device feedback,
        // and broadcasting these as events
        trace!("Starting device feedback handler for motor with node id {node_id}");
        handles.push(task::spawn(feedback::handle_feedback(
            node_id,
            canopen,
            tpdo_mapping,
            event_tx,
            state_feedback_tx,
        )));

        Ok(Cia402Driver {
            node_id,
            cmd_tx,
            event_rx,
            handles,
        })
    }
}

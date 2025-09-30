pub mod command;
pub mod event;
pub mod feedback;
pub mod nmt;
pub mod oms;
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
        startup::motor_startup_task,
        state::{Cia402State, Cia402StateMachine},
    },
    error::DriveError,
    od::oms::Setpoint,
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
    pub fn new(
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

        let handles = Vec::new();

        trace!("Starting NMT State Machine task for motor with node id {node_id}");
        let (nmt_feedback_tx, nmt_feedback_rx) = tokio::sync::mpsc::channel(10);
        handles.push(Nmt::init(node_id, canopen, nmt_feedback_rx));

        let sdo = canopen.get_sdo_client(node_id).expect(format!(
            "Unable to construct SDO client for node id {node_id}"
        ));

        trace!("Starting Startup Task for motor with node id {node_id}");
        let (startup_completed_tx, startup_completed_rx): (
            oneshot::Sender<bool>,
            oneshot::Receiver<bool>,
        ) = tokio::sync::oneshot::channel();
        handles.push(task::spawn(motor_startup_task(
            node_id,
            sdo,
            parameters,
            rpdo_mapping,
            tpdo_mapping,
            event_tx.clone(),
            startup_completed_tx,
        )));

        trace!("Starting Cia402 State Machine task for motor with node id {node_id}");
        let (state_cmd_tx, state_cmd_rx) = tokio::sync::mpsc::channel(10);
        let (state_update_tx, state_update_rx) = tokio::sync::mpsc::channel(10);
        let (state_feedback_tx, state_feedback_rx) = tokio::sync::mpsc::channel(10);
        handles.push(Cia402StateMachine::init(
            node_id,
            j,
            state_cmd_rx,
            state_feedback_rx,
            state_update_tx,
        ));

        trace!("Starting Operational Mode Specific task for motor with node id {node_id}");
        let (setpoint_cmd_tx, setpoint_cmd_rx) = tokio::sync::mpsc::channel(10);
        let (setpoint_update_tx, setpoint_update_rx) = tokio::sync::mpsc::channel(10);
        handles.push(OmsHandler::init(
            node_id,
            canopen,
            setpoint_cmd_rx,
            setpoint_update_tx,
        ));

        let pdo = Pdo::new(canopen, node_id, tpdo_mapping, rpdo_mapping);
        // Spawn command router
        handles.push(task::spawn(command_router(cmd_rx, state_cmd_tx, event_tx)));

        // Spawn feedback task
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

async fn command_router(
    cmd_rx: mpsc::Receiver<MotorCommand>,
    state_transition_tx: tokio::sync::mpsc::Sender<Cia402State>,
    setpoint_tx: tokio::sync::mpsc::Sender<Setpoint>,
) {
    if let Some(cmd) = cmd_rx.recv().await {
        trace!(
            "Command {cmd:?} received, delegating to the cia402 state machine and operation mode specific handler"
        );
        let update = match cmd {
            MotorCommand::Halt => oms.halt_motor(),
            MotorCommand::Enable => {
                state_transition_tx.send(Cia402State::ReadyToSwitchOn).await;
                state_transition_tx.send(Cia402State::SwitchedOn).await;
                state_transition_tx
                    .send(Cia402State::OperationEnabled)
                    .await;
            }
            MotorCommand::Disable => {
                state_transition_tx
                    .send(Cia402State::OperationEnabled)
                    .await;
                state_transition_tx.send(Cia402State::SwitchedOn).await;
                state_transition_tx.send(Cia402State::ReadyToSwitchOn).await;
            }
            MotorCommand::MoveAbsolute { target, velocity } => oms.move_absolute(target, velocity),
            MotorCommand::MoveRelative { delta, velocity } => oms.move_relative(delta, velocity),
            MotorCommand::SetVelocity { velocity } => oms.move_velocity(velocity),
            MotorCommand::SetTorque { torque } => oms.move_torque(torque),
        }
        .await;
    }
}

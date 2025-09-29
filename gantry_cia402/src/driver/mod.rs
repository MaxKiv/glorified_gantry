pub mod command;
pub mod event;
pub mod parametrise;
pub mod state;
pub mod update;
pub mod receiver;

use std::{sync::Arc, time::Duration};

use crate::{
    comms::pdo::{Pdo, mapping::PdoMapping},
    driver::{
        command::MotorCommand,
        event::MotorEvent,
        state::{Cia402State, Cia402StateMachine},
    },
    error::DriveError,
    od::{
        drive_publisher::{Update, publish_updates, write_update},
        oms::{OperationModeSpecificHandler, Setpoint},
    },
};

use anyhow::Result;
use oze_canopen::{canopen::RxMessage, interface::CanOpenInterface, sdo_client::SdoClient};
use tokio::{
    select,
    sync::{Mutex, broadcast, mpsc, watch},
    task::{self, JoinHandle},
    time::Instant,
};
use tracing::*;

pub 

/// CiA-402 driver built on top of a CANopen protocol manager
pub struct Cia402Driver {
    pub node_id: u8,
    cmd_tx: mpsc::Sender<MotorCommand>,
    event_rx: broadcast::Receiver<MotorEvent>,
    state_rx: watch::Receiver<Cia402State>,
    actor_handle: JoinHandle<()>,
}

impl Cia402Driver {
    pub fn new(
        node_id: u8,
        canopen: CanOpenInterface,
        rpdo_mapping: &'static [PdoMapping],
        tpdo_mapping: &'static [PdoMapping],
    ) -> Result<Self, DriveError> {
        // Initialize input interfaces
        let (cmd_tx, cmd_rx): (mpsc::Sender<MotorCommand>, mpsc::Receiver<MotorCommand>) =
            tokio::sync::mpsc::channel(10);

        // Initialize output interfaces
        let (state_tx, state_rx) = tokio::sync::watch::channel(Cia402State::default());
        let (event_tx, event_rx): (
            broadcast::Sender<MotorEvent>,
            broadcast::Receiver<MotorEvent>,
        ) = tokio::sync::broadcast::channel(10);

        // Initialize CANopen communication interface
        let pdo = Pdo::new(canopen, node_id, tpdo_mapping, rpdo_mapping);

        trace!("Parametrizing motor with node id {node_id}");
        // TODO

        // Spawn motor actor task
        // let cmd_handler = task::spawn(handle_cmd(
        //     node_id, canopen, pdo, cmd_rx, state_tx, event_tx,
        // ));

        let frame_handler = task::spawn(receiver::handle_frame(
            node_id, canopen, tpdo_mapping, event_tx, state_tx
        ));

        Ok(Cia402Driver {
            node_id,
            cmd_tx,
            event_rx,
            state_rx,
            actor_handle,
        })
    }
}

async fn run_motor_actor(
    node_id: u8,
    canopen: CanOpenInterface,
    pdo: Pdo,
    cmd_rx: mpsc::Receiver<MotorCommand>,
    state_tx: tokio::sync::watch::Sender<Cia402State>,
    event_tx: broadcast::Sender<MotorEvent>,
) {
    // Initialize Cia402StateMachine handler
    let state_machine = Cia402StateMachine::new();
    let oms_handler = OperationModeSpecificHandler::new(setpoint_tx);

    // Parametrize motor
    trace!("Parametrizing motor with node id {node_id}");
    // TODO!

    // Handle incoming commands and CANopen frames
    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => {
                handle_cmd(cmd, &state_machine, &oms, pdo).await;
            }
            Ok(frame) = canopen.rx.recv() => {
                /* handle frame */
                handle_frame(frame, event_tx, state_tx)
            }
        }
    }
}

async fn handle_cmd(
    cmd_rx: mpsc::Receiver<MotorCommand>,
    cmd: MotorCommand,
    state_machine: Cia402StateMachine,
    oms: OperationModeSpecificHandler,
    pdo: &Pdo,
) -> Result<Update, UpdateError> {
    if let Some(cmd) = cmd_rx.recv().await {
        trace!(
            "Command {cmd} received, delegating to the cia402 state machine and operation mode specific handler"
        );
        let update = match cmd {
            MotorCommand::Halt => oms.halt_motor(),
            MotorCommand::Enable => {
                state_machine
                    .transition_to(Cia402State::OperationEnabled)
                    .await;
            }
            MotorCommand::Disable => {
                state_machine
                    .transition_to(Cia402State::ReadyToSwitchOn)
                    .await;
            }
            MotorCommand::MoveAbsolute { target, velocity } => oms.move_absolute(target, velocity),
            MotorCommand::MoveRelative { delta, velocity } => oms.move_relative(delta, velocity),
            MotorCommand::SetVelocity { velocity } => oms.move_velocity(velocity),
            MotorCommand::SetTorque { torque } => oms.move_torque(torque),
        }
        .await;
    }
}

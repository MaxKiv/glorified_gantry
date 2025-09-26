pub mod feedback;
pub mod lifecycle;
pub mod movement;
pub mod parametrise;

use std::sync::Arc;

use crate::{
    comms::pdo::Pdo,
    error::DriveError,
    od::{
        drive_publisher::publish_updates,
        oms::{OperationModeSpecificHandler, Setpoint},
        state::{Cia402State, Cia402StateMachine},
    },
};

use anyhow::Result;
use oze_canopen::sdo_client::SdoClient;
use tokio::{
    sync::{Mutex, mpsc},
    task::{self, JoinHandle},
};

/// CiA-402 driver built on top of a CANopen protocol manager
pub struct CiA402Drive {
    pub node_id: u8,
    state_machine: Cia402StateMachine,
    oms_handler: OperationModeSpecificHandler,
    sdo: Arc<Mutex<SdoClient>>, // Used for parametrisation
    accessor: Arc<Mutex<Pdo>>,
    handles: Vec<JoinHandle<()>>,
}

impl CiA402Drive {
    pub fn init(
        node_id: u8,
        accessor: Arc<Mutex<Pdo>>,
        sdo: Arc<Mutex<SdoClient>>,
    ) -> Result<Self, DriveError> {
        // TODO: move channels up the stack
        let (state_sender, state_rx): (mpsc::Sender<Cia402State>, mpsc::Receiver<Cia402State>) =
            tokio::sync::mpsc::channel(10);
        let (setpoint_sender, setpoint_rx): (mpsc::Sender<Setpoint>, mpsc::Receiver<Setpoint>) =
            tokio::sync::mpsc::channel(10);

        // Track task handles
        let mut handles = Vec::new();

        // Spawn the CANopenSender and CANopenReceiver tasks
        // These are responsible for handling communication to and from the device
        let pub_handle = task::spawn(publish_updates(accessor.clone(), setpoint_rx, state_rx));
        handles.push(pub_handle);

        Ok(CiA402Drive {
            node_id,
            state_machine: Cia402StateMachine::new(state_sender),
            oms_handler: OperationModeSpecificHandler::new(setpoint_sender),
            sdo,
            handles,
            accessor: accessor.clone(),
        })
    }
}

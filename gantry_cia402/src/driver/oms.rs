use oze_canopen::interface::CanOpenInterface;
use tokio::{
    sync::mpsc::{self},
    task,
};
use tracing::*;

use crate::od::oms::Setpoint;

pub struct OmsHandler {
    node_id: u8,
    canopen: CanOpenInterface,
    setpoint_cmd_rx: mpsc::Receiver<Setpoint>,
    setpoint_update_tx: mpsc::Sender<Setpoint>,
}

impl OmsHandler {
    pub fn init(
        node_id: u8,
        canopen: CanOpenInterface,
        setpoint_cmd_rx: mpsc::Receiver<Setpoint>,
        setpoint_update_tx: mpsc::Sender<Setpoint>,
    ) -> Self {
        let mut oms_handler = Self {
            node_id,
            canopen,
            setpoint_cmd_rx,
            setpoint_update_tx,
        };

        let oms_handle = task::spawn(oms_handler.run());

        oms_handle
    }

    pub async fn run(&mut self) {
        loop {
            // Process Setpoint updates
            if let Some(new_setpoint) = self.setpoint_cmd_rx.recv().await {
                trace!("OMS handler received new setpoint: {new_setpoint:?}",);

                if let Err(err) = self.setpoint_update_tx.send(new_setpoint).await {
                    error!("Failed to send setpoint update to update task: {err}");
                }
            }
        }
    }
}

use gantry_cia402::driver::event::MotorEvent;
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::*;

use crate::{
    axis::{Axis, receiver::AxisEventReceiver},
    event::GantryEvent,
    setpoint::translator::SetpointTranslator,
    spawn_logged,
};

pub struct FeedbackHandle {
    _handles: Vec<Option<JoinHandle<()>>>,
    pub gantry_rx: broadcast::Receiver<GantryEvent>,
}

pub struct FeedbackHandler;

impl FeedbackHandler {
    pub fn init(
        x: Option<AxisEventReceiver>,
        y: Option<AxisEventReceiver>,
        z: Option<AxisEventReceiver>,
        x_translator: Option<SetpointTranslator>,
        y_translator: Option<SetpointTranslator>,
        z_translator: Option<SetpointTranslator>,
    ) -> FeedbackHandle {
        let mut handles = Vec::with_capacity(3);

        // Construct the gantry event broadcast
        let (gantry_tx, gantry_rx) = broadcast::channel(10);

        // A priori clone required structures
        let gantry_tx_x = gantry_tx.clone();
        let gantry_tx_y = gantry_tx.clone();
        let gantry_tx_z = gantry_tx.clone();

        // Spawn a axis feedback handler for each configured gantry axis
        // Spawn X Axis handler, if axis isconfigured
        handles.push(x.map(|x| {
            spawn_logged("FEEDBACK_X", async {
                FeedbackHandler::handle_axis_feedback(x, gantry_tx_x, x_translator.unwrap()).await
            })
        }));

        // Spawn Y Axis handler, if axis is configured
        handles.push(y.map(|y| {
            spawn_logged("FEEDBACK_Y", async {
                FeedbackHandler::handle_axis_feedback(y, gantry_tx_y, y_translator.unwrap()).await
            })
        }));

        // Spawn Z Axis handler, if configured
        handles.push(z.map(|z| {
            spawn_logged("FEEDBACK_Z", async {
                FeedbackHandler::handle_axis_feedback(z, gantry_tx_z, z_translator.unwrap()).await
            })
        }));

        FeedbackHandle {
            _handles: handles,
            gantry_rx,
        }
    }

    pub async fn handle_axis_feedback(
        mut receiver: AxisEventReceiver,
        gantry_tx: broadcast::Sender<GantryEvent>,
        translator: SetpointTranslator,
    ) -> anyhow::Result<()> {
        loop {
            // Handle both master and slave events
            if let Some(slave) = &mut receiver.slave {
                tokio::select! {
                    // Receive motor events from master
                    Ok(event) = receiver.master.recv() => {
                        Self::handle_motor_event(&receiver.axis, event, &gantry_tx, &translator);
                    },
                    // Receive motor events from slave
                    Ok(event) = slave.recv() => {
                        Self::handle_motor_event(&receiver.axis, event, &gantry_tx, &translator);
                    }
                }
            } else {
                // If no slave is configured for this axis: Handle just the master events
                if let Ok(event) = receiver.master.recv().await {
                    trace!(target: "events", "Gantry: motor event received: {:?}", event);
                    Self::handle_motor_event(&receiver.axis, event, &gantry_tx, &translator);
                }
            }
        }
    }

    /// Handles a single received [`MotorEvent`] by translating this into the appropriate
    /// [`GantryEvent`] and broadcasting
    pub fn handle_motor_event(
        axis: &Axis,
        event: MotorEvent,
        gantry_tx: &broadcast::Sender<GantryEvent>,
        translator: &SetpointTranslator,
    ) {
        let gantry_event = GantryEvent::from_motor(axis.clone(), event, translator);

        info!("Sending gantry event: {gantry_event:?}");

        if let Err(e) = gantry_tx.send(gantry_event.clone()) {
            error!("Unable to send Gantry Event: {gantry_event:?}: {e}");
        }
    }
}

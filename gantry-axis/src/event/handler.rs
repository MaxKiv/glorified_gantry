use gantry_cia402::driver::event::MotorEvent;
use oze_canopen::canopen::NodeId;
use tokio::{sync::broadcast, task::JoinSet};
use tracing::*;

use crate::{
    axis::{Axis, receiver::AxisEventReceiver},
    event::{GantryMotorEvent, combiner::EventCombiner},
    setpoint::translator::SetpointTranslator,
};

pub struct FeedbackHandle {
    pub joinset: JoinSet<()>,
    pub gantry_rx: broadcast::Receiver<GantryMotorEvent>,
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
        let mut joinset = JoinSet::new();

        // Construct the gantry event broadcast
        let (gantry_tx, gantry_rx) = broadcast::channel(10);

        // A priori clone required structures
        let gantry_tx_x = gantry_tx.clone();
        let gantry_tx_y = gantry_tx.clone();
        let gantry_tx_z = gantry_tx.clone();

        // Spawn a axis feedback handler for each configured gantry axis
        // Spawn X Axis handler, if axis is configured
        x.map(|x| {
            crate::spawn_logged_joinset(&mut joinset, "FEEDBACK_X", async move {
                FeedbackHandler::handle_axis_feedback(x, gantry_tx_x, x_translator.unwrap()).await
            })
        });

        y.map(|y| {
            crate::spawn_logged_joinset(&mut joinset, "FEEDBACK_Y", async move {
                FeedbackHandler::handle_axis_feedback(y, gantry_tx_y, y_translator.unwrap()).await
            })
        });

        z.map(|z| {
            crate::spawn_logged_joinset(&mut joinset, "FEEDBACK_Z", async move {
                FeedbackHandler::handle_axis_feedback(z, gantry_tx_z, z_translator.unwrap()).await
            })
        });

        FeedbackHandle { joinset, gantry_rx }
    }

    pub async fn handle_axis_feedback(
        mut receiver: AxisEventReceiver,
        gantry_tx: broadcast::Sender<GantryMotorEvent>,
        translator: SetpointTranslator,
    ) -> anyhow::Result<()> {
        let mut combiner = EventCombiner::new_for_axis(
            receiver.axis.clone(),
            receiver.master_id.node_id,
            receiver.slave_id.as_ref().map(|s| s.node_id),
        );

        loop {
            // Handle both master and slave events
            if let Some(slave) = &mut receiver.slave {
                trace!(
                    "Feedback: Axis {:?} has master + slave configuration",
                    receiver.axis.clone()
                );
                tokio::select! {
                    // Receive motor events from master
                    Ok(event) = receiver.master.recv() => {
                        trace!(
                            "Feedback: Axis {:?} received master event: {event:?}",
                            receiver.axis.clone()
                        );

                        Self::handle_motor_event(
                            receiver.axis.clone(),
                            event,
                            receiver.master_id.node_id,
                            &gantry_tx,
                            &translator,
                            &mut combiner
                        );
                    },
                    // Receive motor events from slave
                    Ok(event) = slave.recv() => {
                        if let Some(ref slave_id) = receiver.slave_id {
                            trace!(
                                "Feedback: Axis {:?} received slave event: {event:?}",
                                receiver.axis.clone()
                            );

                            Self::handle_motor_event(receiver.axis.clone(), event, slave_id.node_id,
                                &gantry_tx, &translator, &mut combiner);
                        } else {
                            error!("Received slave event, but no slave id was configured for {:?}", receiver);
                        }
                    }
                }
            } else {
                trace!(
                    "Feedback: Axis {:?} has only master configured, NO slave for this axis",
                    receiver.axis.clone()
                );
                // If no slave is configured for this axis: Handle just the master events
                if let Ok(event) = receiver.master.recv().await {
                    Self::handle_motor_event(
                        receiver.axis.clone(),
                        event,
                        receiver.master_id.node_id,
                        &gantry_tx,
                        &translator,
                        &mut combiner,
                    );
                }
            }
        }
    }

    /// Handles a single received [`MotorEvent`] by translating this into the appropriate
    /// [`GantryEvent`] and broadcasting
    pub fn handle_motor_event(
        axis: Axis,
        event: MotorEvent,
        from_id: NodeId,
        gantry_tx: &broadcast::Sender<GantryMotorEvent>,
        translator: &SetpointTranslator,
        combiner: &mut EventCombiner,
    ) {
        trace!(target: "events", "Gantry: motor event received: {:?}", event);

        // TODO: this unwrap_or defaults to uncombined events for pos/vel/torque feedback;
        let combined_event = combiner.update(from_id, event.clone()).unwrap_or(event);

        let translated_event = translator.translate_motor_event(combined_event);

        let gantry_event = GantryMotorEvent::from_translated(from_id, axis, translated_event);

        info!("Sending gantry event: {gantry_event:?}");

        if let Err(e) = gantry_tx.send(gantry_event.clone()) {
            error!("Unable to send Gantry Event: {gantry_event:?}: {e}");
        }
    }
}

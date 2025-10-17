use std::sync::Arc;

use tokio::{
    sync::{Mutex, broadcast, mpsc},
    task::JoinHandle,
};
use tracing::*;

use crate::{
    comms::pdo::Pdo,
    driver::{event::MotorEvent, oms::setpoint::Setpoint},
    error::DriveError,
};

enum HandshakeState {
    Idle,
    WaitingForAck { setpoint: Setpoint },
}

/// Manages sending of setpoints to the device
/// Also manages the handshake procedure for profile position setpoints
pub struct SetpointManager {
    handshake: HandshakeState,
    new_setpoint_rx: mpsc::Receiver<Setpoint>,
    event_rx: broadcast::Receiver<MotorEvent>,
    pdo: Arc<Mutex<Pdo>>,
}

impl SetpointManager {
    pub fn init(
        event_rx: broadcast::Receiver<MotorEvent>,
        pdo: Arc<Mutex<Pdo>>,
    ) -> (JoinHandle<()>, mpsc::Sender<Setpoint>) {
        let (new_setpoint_tx, new_setpoint_rx) = mpsc::channel(16);

        let mgr = SetpointManager {
            handshake: HandshakeState::Idle,
            new_setpoint_rx,
            event_rx,
            pdo,
        };

        // Run the setpoint manager task
        let handle = tokio::spawn(mgr.run());

        (handle, new_setpoint_tx)
    }

    /// Sends new setpoints to the device
    /// Also handles the handshake procedure for profile position
    async fn run(mut self) {
        loop {
            tokio::select! {
               // Check for handshake events indicating setpoint acknowledge
                Ok(event) = self.event_rx.recv() => {
                    if let MotorEvent::PositionModeFeedback{
                    setpoint_acknowlegded,
                    ..
                    } = event {

                       // Are we shaking hands (aka did we previously set a new setpoint)?
                       if let HandshakeState::WaitingForAck { ref mut setpoint } = self.handshake {
                           // Has the new setpoint been acknowledge by the device?
                           if setpoint_acknowlegded {
                               trace!(
                                   "Setpoint manager observed handshake / Setpoint Acknowledge for previously sent setpoint {setpoint:?}"
                               );

                               // Clear CW bit 4 indicating setpoint acknowledge
                               setpoint.acknowledge_setpoint_received();

                               // Complete acknowledge procedure by writing the updated setpoint to the device
                               if let Err(err) = self.pdo.lock().await.write_setpoint(setpoint).await {
                                   error!(
                                   "Setpoint manager unable to complete setpoint acknowledge procedure by writing new setpoint (sans cw bit 4) to device: {err}"
                                   );
                               }

                               // Setpoint acknowledged
                               self.handshake = HandshakeState::Idle;
                           }
                       }
                   }
               }

                // A new setpoint arrives, write it to the device
                // Also restart the handshake procedure if required
               Some(new_setpoint) = self.new_setpoint_rx.recv() => {
                    trace!("Setpoint manager writing new setpoint {new_setpoint:?}");

                   if let Err(err) = self.pdo.lock().await.write_setpoint(&new_setpoint).await {
                       error!(
                           "Setpoint manager unable send new setpoint to device: {err}"
                       );
                    }

                    // Start handshake procedure if required
                   if Self::handshake_required_for_setpoint(&new_setpoint) {
                        trace!("Setpoint manager requires handshake for new setpoing {new_setpoint:?}");
                       self.handshake = HandshakeState::WaitingForAck{setpoint: new_setpoint};
                   }
               }

            }
        }
    }

    /// Is a handshake required for this setpoint/mode?
    fn handshake_required_for_setpoint(setpoint: &Setpoint) -> bool {
        matches!(setpoint, Setpoint::ProfilePosition(_))
    }

    /// Request the setpoint manager to write a new setpoint to the device
    /// Also starts a handshake procedure if required
    pub async fn write_new_setpoint(
        new_setpoint_tx: &mpsc::Sender<Setpoint>,
        setpoint: Setpoint,
    ) -> Result<(), DriveError> {
        trace!("Sending new setpoint to setpoint manager: {setpoint:?}");

        new_setpoint_tx
            .send(setpoint.clone())
            .await
            .map_err(|e| DriveError::NewSetpointSendError(setpoint, e))
    }
}

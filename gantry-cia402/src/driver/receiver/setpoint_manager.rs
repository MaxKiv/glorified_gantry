use std::{sync::Arc, time::Duration};

use oze_canopen::{canopen::NodeId, sdo_client::SdoClient};
use tokio::{
    sync::{Mutex, broadcast, mpsc, watch},
    task::JoinHandle,
    time::Instant,
};
use tracing::*;

use crate::{
    comms::pdo::{cmd::PdoCommand, mapping::custom::CUSTOM_PDOS},
    driver::{
        cyclic::CyclicSynchronousMode,
        event::MotorEvent,
        nmt::{NmtState, set_to_nmt_state},
        oms::setpoint::Setpoint,
        startup::pdo_mapping::configure_pdo_mappings,
    },
    error::DriveError,
};

pub const SYNC_CYCLE_ERROR_WINDOW: Duration = Duration::from_millis(10);

enum HandshakeState {
    Idle,
    WaitingForAck { setpoint: Setpoint },
}

#[derive(Clone, Debug)]
pub enum SetpointManagerModeTypes {
    Default,
    CyclicSynchronous(CyclicSynchronousMode),
}

/// Manages sending of setpoints to the device. This is done once on change for
/// Default mode and continously on every SYNC cycle for CyclicSynchronous Modes
/// Also manages the handshake procedure for profile position setpoints
pub struct SetpointManager {
    node_id: NodeId,
    handshake: HandshakeState,
    new_setpoint_rx: mpsc::Receiver<Setpoint>,
    cs_mode_rx: watch::Receiver<SetpointManagerModeTypes>,
    event_rx: broadcast::Receiver<MotorEvent>,
    pdo_tx: mpsc::Sender<PdoCommand>,
    sync_rx: broadcast::Receiver<Instant>,
    current_setpoint: Option<Setpoint>,
    mode: SetpointManagerModeTypes,
    last_sync: Option<Instant>,
}

impl SetpointManager {
    pub fn init(
        node_id: NodeId,
        event_rx: broadcast::Receiver<MotorEvent>,
        pdo_tx: mpsc::Sender<PdoCommand>,
        sync_rx: broadcast::Receiver<Instant>,
    ) -> (
        JoinHandle<()>,
        mpsc::Sender<Setpoint>,
        watch::Sender<SetpointManagerModeTypes>,
    ) {
        let (new_setpoint_tx, new_setpoint_rx) = mpsc::channel(16);
        let (cs_mode_tx, mut cs_mode_rx) = watch::channel(SetpointManagerModeTypes::Default);
        let mode = cs_mode_rx.borrow_and_update().clone();

        let mgr = SetpointManager {
            handshake: HandshakeState::Idle,
            new_setpoint_rx,
            event_rx,
            pdo_tx,
            sync_rx,
            mode,
            cs_mode_rx,
            current_setpoint: None,
            last_sync: None,
            node_id,
        };

        // Run the setpoint manager task
        let handle = tokio::spawn(mgr.run());

        (handle, new_setpoint_tx, cs_mode_tx)
    }

    /// Sends new setpoints to the device using [`Pdo`] as transport layer
    /// Handles the handshake procedure for profile position
    /// And Sync callbacks required for any CyclicSynchronous mode
    async fn run(mut self) {
        loop {
            tokio::select! {
               // A new setpoint arrives, write it to the device
               // Also restart the handshake procedure if required
               Some(new_setpoint) = self.new_setpoint_rx.recv() => {
                   trace!("Setpoint manager writing new setpoint {new_setpoint:?}");

                   // Track current setpoint
                   self.current_setpoint = Some(new_setpoint.clone());

                   // Write new setpoints to the device in default mode
                   if let SetpointManagerModeTypes::Default = self.mode {
                       if let Err(err) = self.pdo_tx.send(PdoCommand::WriteSetpoint(new_setpoint.clone())).await {
                           error!("Setpoint manager unable send new setpoint to device: {err}");
                       }

                       // Start handshake procedure if required
                       if Self::handshake_required_for_setpoint(&new_setpoint) {
                           warn!("xxx {} Setpoint manager requires handshake for new setpoint
                               {new_setpoint:?}", self.node_id);
                           self.handshake = HandshakeState::WaitingForAck{setpoint: new_setpoint};
                       }
                   }
               }

               // Check for handshake events indicating setpoint acknowledge
               Ok(event) = self.event_rx.recv() => {
                   if let MotorEvent::PositionModeFeedback{
                   setpoint_acknowlegded,
                   ..
                   } = event {
                       if setpoint_acknowlegded {
                          warn!(
                              "xxx {} Setpoint manager observed a handshake",
                              self.node_id
                          );
                       }

                      // Are we shaking hands (aka did we previously set a new setpoint)?
                      if let HandshakeState::WaitingForAck { ref mut setpoint } = self.handshake {
                          // Has the new setpoint been acknowledge by the device?
                          if setpoint_acknowlegded {
                              warn!(
                                  "xxx {} handshake confirmed",
                                  self.node_id
                              );

                              // Clear CW bit 4 indicating setpoint acknowledge
                              setpoint.acknowledge_setpoint_received();

                              // Complete acknowledge procedure by writing the updated setpoint to the device
                              if let Err(err) = self.pdo_tx.send(PdoCommand::WriteSetpoint(setpoint.clone())).await {
                                  warn!("xxx {} Setpoint Manager unable to complete setpoint
                                      handshake procedure: {err}", self.node_id);
                              }

                              // Setpoint acknowledged
                              self.handshake = HandshakeState::Idle;
                          }
                      }
                  }
               }

               // A mode change arrived
               Ok(_) = self.cs_mode_rx.changed() => {
                   // Update our current mode to the one requested
                   {
                       self.mode = self.cs_mode_rx.borrow_and_update().clone();
                   }
                   // Seperate Mode switch is required for CyclicSynchronous Modes
                   if let SetpointManagerModeTypes::CyclicSynchronous(ref mode) = self.mode
                       && let Err(err) =
                    self.pdo_tx.send(PdoCommand::SwitchToCyclicSynchronousMode(mode.clone())).await {
                           error!("Setpoint Manager unable to switch to CyclicSynchronousMode: {mode:?} - {err}");
                       }
               }

               // A SYNC Master notifies us that it has just posted a SYNC on the bus
               Ok(this_sync) = self.sync_rx.recv() => {
                   // We only care about SYNC stuff if we are in a Cycic Synchronous mode
                   if let SetpointManagerModeTypes::CyclicSynchronous(_) = self.mode {
                       // Send the current setpoint to the device
                       if let Some(setpoint) = &self.current_setpoint {
                           if setpoint.is_cyclic_synchronous() {
                               error!("Setpoint manager attempts to write non-cyclic setpoint: {setpoint:?} on SYNC for Mode: {:?}", self.mode);
                           } else if let Err(err) = self.pdo_tx.send(PdoCommand::WriteSetpoint(setpoint.clone())).await {
                               error!("Setpoint manager unable send new setpoint to device: {err}");
                           }
                       }
                   }

                   // Check if SYNC cycle timing is adequate
                   if let Some(last_sync) = self.last_sync {
                       if this_sync - last_sync > SYNC_CYCLE_ERROR_WINDOW {
                          error!("this sync: {this_sync:?}, last sync {last_sync:?} -> SYNC Cycle time is too slow!");
                       }
                       self.last_sync = Some(this_sync);
                   }
               }

            }
        }
    }

    /// Is a handshake required for this setpoint/mode?
    fn handshake_required_for_setpoint(setpoint: &Setpoint) -> bool {
        matches!(setpoint, Setpoint::ProfilePosition(_))
    }

    /// Switch to Cyclic Synchronous Mode
    /// In this mode Setpoint Manager expects a bus master to produce a regular SYNC
    /// On every SYNC the current setpoint is written to the device
    pub async fn enable_cyclic_synchronous_mode(
        cs_mode_tx: &watch::Sender<SetpointManagerModeTypes>,
        mode: CyclicSynchronousMode,
        nmt_tx: &mpsc::Sender<NmtState>,
        event_rx: broadcast::Receiver<MotorEvent>,
        sdo: Arc<Mutex<SdoClient>>,
        node_id: u8,
    ) -> Result<(), DriveError> {
        trace!("Enabling Cyclic Synchronous Mode for device {node_id} - NMT PRE-OP");
        // Put the drive in NMT PreOperational, required for parametrisation & pdo mapping
        set_to_nmt_state(NmtState::PreOperational, &nmt_tx, event_rx.resubscribe()).await?;

        // Reconfigure pdo mappings
        trace!("Enabling Cyclic Synchronous Mode for device {node_id} - Reconfiguring RPDOS");
        configure_pdo_mappings(node_id, sdo.clone(), CUSTOM_PDOS.rpdos).await;
        trace!("Enabling Cyclic Synchronous Mode for device {node_id} - Reconfiguring TPDOS");
        configure_pdo_mappings(node_id, sdo.clone(), CUSTOM_PDOS.tpdos).await;

        // Set the drive into NMT Operational again
        trace!("Enabling Cyclic Synchronous Mode for device {node_id} - NMT Operational");
        set_to_nmt_state(NmtState::Operational, &nmt_tx, event_rx.resubscribe()).await?;

        trace!(
            "Enabling Cyclic Synchronous Mode for device {node_id} - Setting Setpoint Manager mode to {:?}",
            mode
        );

        // Initiate setpoing manager CyclicSynchronous Mode operation
        cs_mode_tx
            .send(SetpointManagerModeTypes::CyclicSynchronous(mode))
            .map_err(DriveError::ModeSwitchError)
    }

    /// Disable Cyclic Synchronous Mode
    /// The setpoint manager will stop writing the current setpoint to the device on SYNC
    pub async fn disable_cyclic_synchronous_mode(
        cs_mode_tx: &watch::Sender<SetpointManagerModeTypes>,
        nmt_tx: &mpsc::Sender<NmtState>,
        event_rx: broadcast::Receiver<MotorEvent>,
        sdo: Arc<Mutex<SdoClient>>,
        node_id: u8,
    ) -> Result<(), DriveError> {
        trace!("Disabling Cyclic Synchronous Mode for device {node_id} - NMT PRE-OP");
        // Put the drive in NMT PreOperational, required for parametrisation & pdo mapping
        set_to_nmt_state(NmtState::PreOperational, &nmt_tx, event_rx.resubscribe()).await?;

        // Reconfigure pdo mappings
        trace!("Disabling Cyclic Synchronous Mode for device {node_id} - Reconfiguring RPDOS");
        configure_pdo_mappings(node_id, sdo.clone(), CUSTOM_PDOS.rpdos).await;
        trace!("Disabling Cyclic Synchronous Mode for device {node_id} - Reconfiguring TPDOS");
        configure_pdo_mappings(node_id, sdo.clone(), CUSTOM_PDOS.tpdos).await;

        // Set the drive into NMT Operational again
        trace!("Disabling Cyclic Synchronous Mode for device {node_id} - NMT Operational");
        set_to_nmt_state(NmtState::Operational, &nmt_tx, event_rx.resubscribe()).await?;

        // Initiate setpoint manager default mode operation
        trace!(
            "Disabling Cyclic Synchronous Mode for device {node_id} - Setting Setpoint Manager mode to Default"
        );
        cs_mode_tx
            .send(SetpointManagerModeTypes::Default)
            .map_err(DriveError::ModeSwitchError)
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

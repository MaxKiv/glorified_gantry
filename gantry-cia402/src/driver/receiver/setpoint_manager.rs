use std::{sync::Arc, time::Duration};

use oze_canopen::{canopen::NodeId, sdo_client::SdoClient};
use tokio::{
    sync::{Mutex, broadcast, mpsc, watch},
    task::AbortHandle,
    time::Instant,
};
use tracing::*;

use crate::{
    comms::pdo::{
        cmd::PdoCommand,
        mapping::{PdoSet, default::DEFAULT_PDOS, table::DEFAULT_PDO_TABLE},
    },
    driver::{
        cyclic::CyclicSynchronousMode,
        event::MotorEvent,
        nmt::{NmtState, set_to_nmt_state},
        oms::{OperationMode, setpoint::Setpoint, torque::TorqueSetpoint},
        startup::pdo_mapping::configure_pdo_mappings,
    },
    error::DriveError,
};

pub const SYNC_PERIOD_ALLOWED_EPSILON: Duration = Duration::from_micros(1);

enum SyncState {
    Desynchronised,
    Synchronised,
}

enum HandshakeState {
    Idle,
    WaitingForAck { setpoint: Setpoint },
}

#[derive(Clone, Debug)]
pub enum SetpointManagerModeTypes {
    NonCyclic,
    CyclicSynchronous(CyclicSynchronousMode),
}

/// Manages sending of setpoints to the device. This is done once on change for
/// Default mode and continously on every SYNC cycle for CyclicSynchronous Modes
/// Also manages the handshake procedure for profile position setpoints
pub struct SetpointManager {
    node_id: NodeId,
    handshake: HandshakeState,
    new_setpoint_rx: mpsc::Receiver<Setpoint>,
    event_rx: broadcast::Receiver<MotorEvent>,
    pdo_tx: mpsc::Sender<PdoCommand>,
    current_setpoint: Option<Setpoint>,
    mode_transition_rx: watch::Receiver<OperationMode>,
    mode: OperationMode,
    last_sync: Option<Instant>,
    sync_rx: broadcast::Receiver<Instant>,
    sync_period: Duration,
    sync_state: SyncState,
    nmt_tx: mpsc::Sender<NmtState>,
    sdo: Arc<Mutex<SdoClient>>,
    current_pdo_set_tx: watch::Sender<&'static PdoSet>,
}

impl SetpointManager {
    pub fn init(
        node_id: NodeId,
        event_rx: broadcast::Receiver<MotorEvent>,
        pdo_tx: mpsc::Sender<PdoCommand>,
        sync_rx: broadcast::Receiver<Instant>,
        sync_period: Duration,
        set: &mut tokio::task::JoinSet<()>,
        nmt_tx: mpsc::Sender<NmtState>,
        sdo: Arc<Mutex<SdoClient>>,
        current_pdo_set_tx: watch::Sender<&'static PdoSet>,
    ) -> (
        AbortHandle,
        mpsc::Sender<Setpoint>,
        watch::Sender<OperationMode>,
    ) {
        let (new_setpoint_tx, new_setpoint_rx) = mpsc::channel(16);
        let (mode_transition_tx, mut mode_transition_rx) = watch::channel(OperationMode::Homing);
        let mode = mode_transition_rx.borrow_and_update().clone();

        let mgr = SetpointManager {
            handshake: HandshakeState::Idle,
            sync_state: SyncState::Desynchronised,
            new_setpoint_rx,
            event_rx,
            pdo_tx,
            sync_rx,
            mode,
            mode_transition_rx,
            current_setpoint: None,
            last_sync: None,
            node_id,
            sync_period,
            nmt_tx,
            sdo,
            current_pdo_set_tx,
        };

        // Run the setpoint manager task
        let handle = set.spawn(mgr.run());

        (handle, new_setpoint_tx, mode_transition_tx)
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

                    // Write new setpoints to the device in noncyclic mode
                    // NOTE: cyclic mode setpoints are send after SYNC arrives, see below
                    if self.in_non_cyclic_mode() {
                        if let Err(err) = self.pdo_tx.send(PdoCommand::WriteSetpoint(new_setpoint.clone())).await {
                            error!("Setpoint manager unable send new setpoint to device: {err}");
                        }

                        // Start handshake procedure if required
                        if Self::handshake_required_for_setpoint(&new_setpoint) {
                            trace!("SetpointManager for motor {} requires handshake for new setpoint {new_setpoint:?}", self.node_id);
                            self.handshake = HandshakeState::WaitingForAck{setpoint: new_setpoint};
                        }
                    }
                }

                // Check for handshake events indicating setpoint acknowledge
                Ok(event) = self.event_rx.recv() => {
                    match event {
                        MotorEvent::PositionModeFeedback {
                            setpoint_acknowlegded,
                            ..
                        } => {
                            if setpoint_acknowlegded {
                                trace!(
                                    "Setpoint manager for node {} observed a handshake",
                                    self.node_id
                                );
                            }

                            // Are we shaking hands (aka did we just set a new Profile Position or Homing setpoint)?
                            if let HandshakeState::WaitingForAck { ref mut setpoint } = self.handshake {
                                // Has the new setpoint been acknowledge by the device?
                                if setpoint_acknowlegded {
                                    trace!(
                                        "Setpoint Manager for motor {} handshake confirmed",
                                        self.node_id
                                    );

                                    // Clear CW bit 4 indicating setpoint acknowledge
                                    setpoint.acknowledge_setpoint_received();

                                    let flag_update_cmd = match setpoint {
                                        Setpoint::ProfilePosition(position_setpoint) => {
                                            Some(PdoCommand::UpdatePositionSetpointFlags(position_setpoint.flags))
                                        },
                                        Setpoint::Home(homing_setpoint) => {
                                            Some(PdoCommand::UpdateHomingSetpointFlags(homing_setpoint.flags))
                                        },
                                        _ => {
                                            None
                                        }
                                    };

                                    if let Some(cmd) = flag_update_cmd {
                                        if let Err(err) =
                                            self.pdo_tx.send(cmd).await {
                                                warn!("
                                                    Setpoint Manager for motor {} is unable to complete setpoint handshake procedure: {err}"
                                                    , self.node_id
                                                );
                                        }
                                    }

                                    // Setpoint acknowledged
                                    self.handshake = HandshakeState::Idle;
                                }
                            }
                        },

                        // Update Sync State if drive tells us it is in sync
                        MotorEvent::CyclicPositionModeFeedback { device_in_sync, is_following_target, .. } => {
                            if device_in_sync && is_following_target {
                                if self.mode == OperationMode::CyclicSynchronousPosition {
                                    self.sync_state = SyncState::Synchronised;
                                }
                            }
                        }
                        MotorEvent::CyclicVelocityModeFeedback { device_in_sync, is_following_target } => {
                            if device_in_sync && is_following_target {
                                if self.mode == OperationMode::CyclicSynchronousVelocity {
                                    self.sync_state = SyncState::Synchronised;
                                }
                            }
                        },
                        MotorEvent::CyclicTorqueModeFeedback { device_in_sync, is_following_target } => {
                            if device_in_sync && is_following_target {
                                if self.mode == OperationMode::CyclicSynchronousTorque {
                                    self.sync_state = SyncState::Synchronised;
                                }
                            }
                        },
                        _ => {},
                    }
                }

                // A mode change arrived
                Ok(_) = self.mode_transition_rx.changed() => {
                    let mode = {
                        self.mode_transition_rx.borrow_and_update().clone()
                    };

                    if let Err(err) = self.switch_to_new_operating_mode(mode).await {
                        error!("Setpoint manager unable to switch to new {mode:?} - {err}");
                    }
                }

                // A SYNC Master notifies us that it has just posted a SYNC on the bus
                Ok(this_sync) = self.sync_rx.recv() => {
                    // We only care about SYNC stuff if we are in a Cyclic Synchronous mode
                    if self.in_cyclic_mode() {
                        // What is our current synchronisation state?
                        match self.sync_state {
                            SyncState::Desynchronised => {
                                // Send default/safe cyclic mode setpoints while waiting for drive
                                // synchronisation
                                // => Wait for SW bit 8 & 12
                                self.current_setpoint =
                                    Some(Setpoint::get_safe_setpoint_for_mode(self.mode));
                                self.send_current_setpoint().await;
                            }
                            SyncState::Synchronised => {
                                // drive synchronised, pass on user setpoint for this cycle to drive
                                // Check if the current setpoint is a valid cyclic setpoint
                                // Set to safe setpoint if not
                                match &self.current_setpoint {
                                    Some(sp) => {
                                        if !sp.is_cyclic_synchronous(){
                                            self.current_setpoint = Some(Setpoint::get_safe_setpoint_for_mode(self.mode));
                                        }
                                    },
                                    None => {
                                        self.current_setpoint = Some(Setpoint::get_safe_setpoint_for_mode(self.mode));
                                    },
                                }

                                // Send current setpoint to drive
                                self.send_current_setpoint().await;
                            }
                        };
                    }

                    // Check if SYNC cycle timing is adequate
                    if let Some(last_sync) = self.last_sync {
                        let curr_sync_period = this_sync - last_sync;
                        let curr_jitter = curr_sync_period - self.sync_period;
                        error!("SYNC: period: {curr_sync_period:?} - jitter: {curr_jitter:?}");
                        // if (curr_sync_period > self.sync_period + SYNC_PERIOD_ALLOWED_EPSILON) || (curr_sync_period < self.sync_period - SYNC_PERIOD_ALLOWED_EPSILON){
                        //     error!("SYNC Period: {curr_sync_period:?} outside allowed epsilon of {SYNC_PERIOD_ALLOWED_EPSILON:?} -> SYNC Cycle time is too slow!");
                        // }
                    }
                    self.last_sync = Some(this_sync);
                }

            }
        }
    }

    /// Is a handshake required for this setpoint/mode?
    fn handshake_required_for_setpoint(setpoint: &Setpoint) -> bool {
        matches!(setpoint, Setpoint::ProfilePosition(_))
    }

    /// Send current setpoint to drive
    async fn send_current_setpoint(&self) -> Result<(), DriveError> {
        let setpoint = if let Some(setpoint) = &self.current_setpoint {
            setpoint.clone()
        } else {
            Setpoint::get_safe_setpoint_for_mode(self.mode)
        };

        let pdo_cmd = PdoCommand::WriteSetpoint(setpoint);
        let out = self
            .pdo_tx
            .send(pdo_cmd.clone())
            .await
            .map_err(|_| DriveError::PdoCommandError(pdo_cmd));
        if let Err(err) = &out {
            error!("Setpoint manager unable send new setpoint to device: {err}");
        }

        out
    }

    /// Is the setpoint manager in a Cyclic Synchronous mode?
    fn in_cyclic_mode(&self) -> bool {
        self.mode.is_cyclic_synchronous()
    }
    fn in_non_cyclic_mode(&self) -> bool {
        !self.in_cyclic_mode()
    }

    async fn switch_to_new_operating_mode(
        &mut self,
        new_mode: OperationMode,
    ) -> Result<(), DriveError> {
        if self.in_non_cyclic_mode() && new_mode.is_cyclic_synchronous() {
            // Switching from cyclic -> non cyclic requires PDO mapping change
            self.remap_drive_pdoset_for_new_operationmode(new_mode)
                .await?;
        } else if self.in_cyclic_mode() && new_mode != self.mode {
            // Switching between cyclic modes requries PDO mapping change
            self.remap_drive_pdoset_for_new_operationmode(new_mode)
                .await?;
        }

        // Update setpoint manager mode
        self.mode = new_mode;

        Ok(())
    }

    /// Switch to Cyclic Synchronous Mode
    /// In this mode Setpoint Manager expects a bus master to produce a regular SYNC
    /// On every SYNC the current setpoint is written to the device
    pub async fn enable_cyclic_synchronous_mode(
        mode_transition_tx: &watch::Sender<OperationMode>,
        mode: CyclicSynchronousMode,
        node_id: u8,
    ) -> Result<(), DriveError> {
        trace!(
            "Enabling Cyclic Synchronous Mode for device {node_id} - Setting Setpoint Manager mode to {:?}",
            mode
        );

        let mode = mode.try_into().unwrap();
        // Initiate setpoing manager CyclicSynchronous Mode operation
        mode_transition_tx
            .send(mode)
            .map_err(DriveError::ModeSwitchError)
    }

    /// Disable Cyclic Synchronous Mode
    /// The setpoint manager will stop writing the current setpoint to the device on SYNC
    pub async fn disable_cyclic_synchronous_mode(
        cs_mode_tx: &watch::Sender<OperationMode>,
        node_id: u8,
    ) -> Result<(), DriveError> {
        // Initiate setpoint manager noncyclic mode operation
        trace!(
            "Disabling Cyclic Synchronous Mode for device {node_id} - Setting Setpoint Manager mode
            to noncyclic - Profile Torque"
        );
        const DEFAULT_MODE: OperationMode = OperationMode::ProfileTorque;
        cs_mode_tx
            .send(DEFAULT_MODE)
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

    async fn remap_drive_pdoset_for_new_operationmode(
        &self,
        new_mode: OperationMode,
    ) -> Result<(), DriveError> {
        // TODO: gracefully drop cia402 sm into ReadyToSwitchOn before NMT pre-op?

        // Setpoint manager in default mode, mode switch to cyclic requested
        trace!(
            "Enabling Cyclic Synchronous Mode for device {} - NMT PRE-OP",
            self.node_id
        );
        // Put the drive in NMT PreOperational, required for parametrisation & pdo mapping
        set_to_nmt_state(
            NmtState::PreOperational,
            &self.nmt_tx,
            self.event_rx.resubscribe(),
        )
        .await?;

        let new_pdoset = DEFAULT_PDO_TABLE.get_pdoset_for_operationmode(new_mode);

        // Reconfigure pdo mappings
        trace!(
            "Enabling Cyclic Synchronous Mode for device {} - Reconfiguring RPDOS",
            self.node_id
        );
        let rpdos = new_pdoset.rpdos;
        configure_pdo_mappings(self.node_id, self.sdo.clone(), rpdos).await?;

        trace!(
            "Enabling Cyclic Synchronous Mode for device {} - Reconfiguring TPDOS",
            self.node_id
        );
        let tpdos = new_pdoset.tpdos;
        configure_pdo_mappings(self.node_id, self.sdo.clone(), tpdos).await?;

        // Indicate succesful PdoSet switch
        self.current_pdo_set_tx.send_replace(new_pdoset);

        // Master should have started SYNC around here

        // Set the drive into NMT Operational again
        trace!(
            "Enabling Cyclic Synchronous Mode for device {} - NMT Operational",
            self.node_id
        );
        set_to_nmt_state(
            NmtState::Operational,
            &self.nmt_tx,
            self.event_rx.resubscribe(),
        )
        .await?;

        let pdo_cmd = PdoCommand::SwitchToCyclicSynchronousMode(self.mode.try_into().unwrap());
        self.pdo_tx
            .send(pdo_cmd.clone())
            .await
            .map_err(|_| DriveError::PdoCommandError(pdo_cmd))?;

        Ok(())
    }

    async fn switch_to_noncyclic_mode(
        &self,
        default_mode: OperationMode,
    ) -> Result<(), DriveError> {
        // TODO: gracefully drop cia402 sm into ReadyToSwitchOn before NMT pre-op?
        let node_id = self.node_id;

        trace!("Disabling Cyclic Synchronous Mode for device {node_id} - NMT PRE-OP");
        // Put the drive in NMT PreOperational, required for parametrisation & pdo mapping
        set_to_nmt_state(
            NmtState::PreOperational,
            &self.nmt_tx,
            self.event_rx.resubscribe(),
        )
        .await?;

        // Reconfigure pdo mappings
        trace!("Disabling Cyclic Synchronous Mode for device {node_id} - Reconfiguring RPDOS");
        configure_pdo_mappings(node_id, self.sdo.clone(), DEFAULT_PDOS.rpdos).await?;
        trace!("Disabling Cyclic Synchronous Mode for device {node_id} - Reconfiguring TPDOS");
        configure_pdo_mappings(node_id, self.sdo.clone(), DEFAULT_PDOS.tpdos).await?;

        // Set the drive into NMT Operational again
        trace!("Disabling Cyclic Synchronous Mode for device {node_id} - NMT Operational");
        set_to_nmt_state(
            NmtState::Operational,
            &self.nmt_tx,
            self.event_rx.resubscribe(),
        )
        .await?;

        let sp = match default_mode {
            OperationMode::ProfileTorque => {
                Setpoint::ProfileTorque(TorqueSetpoint { target_torque: 0 })
            }
            _ => {
                error!("When switching away from cyclic mode only ProfileTorque is allowed");
                return Err(DriveError::ViolatedInvariant(
                    "When switching away from cyclic mode only ProfileTorque is allowed"
                        .to_string(),
                ));
            }
        };

        let pdo_cmd = PdoCommand::WriteSetpoint(sp);
        self.pdo_tx
            .send(pdo_cmd.clone())
            .await
            .map_err(|_| DriveError::PdoCommandError(pdo_cmd))?;

        Ok(())
    }
}

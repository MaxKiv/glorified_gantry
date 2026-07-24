use oze_canopen::interface::CanOpenInterface;
use tokio::{sync::broadcast, time::Instant};

use crate::{
    comms::{
        pdo::mapping::{PDOSet, custom::CUSTOM_PDOS, minimal::MINIMAL_CYCLIC_SYNCHRONOUS_PDO_SET},
        sdo::SdoAction,
    },
    driver::{
        AxisMaster, AxisSlave, Cia402Driver, Standalone, command::MotorCommand,
        identifier::Cia402Identifier, startup::params::TEST_PARAMS,
    },
    error::InitialisationError,
};

pub const DEFAULT_PDO_SET: &PDOSet = &CUSTOM_PDOS;
pub const MINIMAL_PDO_SET: &PDOSet = &MINIMAL_CYCLIC_SYNCHRONOUS_PDO_SET;

// Typestate structs
/// No CANOpen configured yet
pub struct NoCanOpen;
/// With valid CANOpen configuration
pub struct HasCanOpen;

/// No T/RPDO mapping configured yet
pub struct NoMapping;
/// With valid T/RPDO mapping configured yet
pub struct HasMapping;

/// No SYNC receiver configured yet
pub struct NoSyncReceiver;
/// With valid SYNC receiver
pub struct HasSyncReceiver;

/// Typestate Builder for the Cia402Driver
pub struct Cia402DriverBuilder<C, M, S, Mode> {
    identifier: Cia402Identifier,
    canopen: Option<CanOpenInterface>,
    parameters: Option<&'static [SdoAction<'static>]>,
    default_pdo_set: Option<&'static PDOSet>,
    minimal_pdo_set: Option<&'static PDOSet>,
    sync_rx: Option<broadcast::Receiver<Instant>>,
    cmd_tx: Option<broadcast::Sender<MotorCommand>>,
    cmd_rx: Option<broadcast::Receiver<MotorCommand>>,
    _canopen: std::marker::PhantomData<C>,
    _mapping: std::marker::PhantomData<M>,
    _sync: std::marker::PhantomData<S>,
    _mode: std::marker::PhantomData<Mode>,
}

impl Cia402DriverBuilder<NoCanOpen, NoMapping, NoSyncReceiver, Standalone> {
    /// Start building a new Cia402Driver
    pub fn new(identifier: Cia402Identifier) -> Self {
        Self {
            identifier,
            canopen: None,
            parameters: None,
            default_pdo_set: None,
            minimal_pdo_set: None,
            sync_rx: None,
            cmd_tx: None,
            cmd_rx: None,
            _canopen: std::marker::PhantomData,
            _mapping: std::marker::PhantomData,
            _sync: std::marker::PhantomData,
            _mode: std::marker::PhantomData,
        }
    }
}

// Required Cia402Driver configuration
impl<M, S, Mode> Cia402DriverBuilder<NoCanOpen, M, S, Mode> {
    /// Configure the Cia402Driver with a CANOpen interface
    pub fn with_canopen(
        self,
        iface: CanOpenInterface,
    ) -> Cia402DriverBuilder<HasCanOpen, M, S, Mode> {
        Cia402DriverBuilder {
            identifier: self.identifier,
            canopen: Some(iface),
            parameters: self.parameters,
            minimal_pdo_set: self.minimal_pdo_set,
            default_pdo_set: self.default_pdo_set,
            sync_rx: self.sync_rx,
            cmd_tx: self.cmd_tx,
            cmd_rx: self.cmd_rx,
            _canopen: std::marker::PhantomData,
            _mapping: self._mapping,
            _sync: self._sync,
            _mode: self._mode,
        }
    }
}

// Required Cia402Driver configuration
impl<C, S, Mode> Cia402DriverBuilder<C, NoMapping, S, Mode> {
    /// Configure the Cia402Driver with a T/RPDO mapping
    pub fn with_default_pdo_mappings(self) -> Cia402DriverBuilder<C, HasMapping, S, Mode> {
        let default_pdo_set = DEFAULT_PDO_SET;
        let minimal_pdo_set = MINIMAL_PDO_SET;

        self.with_pdo_mappings(default_pdo_set, minimal_pdo_set)
    }

    fn with_pdo_mappings(
        self,
        default_pdo_set: &'static PDOSet,
        minimal_pdo_set: &'static PDOSet,
    ) -> Cia402DriverBuilder<C, HasMapping, S, Mode> {
        if default_pdo_set.contains_default_rpdo() && minimal_pdo_set.contains_minimal_rpdo() {
            Cia402DriverBuilder {
                identifier: self.identifier,
                canopen: self.canopen,
                parameters: self.parameters,
                default_pdo_set: Some(default_pdo_set),
                minimal_pdo_set: Some(minimal_pdo_set),
                sync_rx: self.sync_rx,
                cmd_tx: self.cmd_tx,
                cmd_rx: self.cmd_rx,
                _canopen: self._canopen,
                _mapping: std::marker::PhantomData,
                _sync: self._sync,
                _mode: self._mode,
            }
        } else {
            panic!(
                "Building Cia402Driver with incorrect default ({:?}) and minimal ({:?}) PDO set",
                default_pdo_set, minimal_pdo_set
            )
        }
    }
}

// Required Cia402Driver configuration
impl<C, M, Mode> Cia402DriverBuilder<C, M, NoSyncReceiver, Mode> {
    /// Configure the Cia402Driver with a T/RPDO mapping
    pub fn with_sync_receiver(
        self,
        sync_rx: broadcast::Receiver<Instant>,
    ) -> Cia402DriverBuilder<C, M, HasSyncReceiver, Mode> {
        Cia402DriverBuilder {
            identifier: self.identifier,
            canopen: self.canopen,
            parameters: self.parameters,
            minimal_pdo_set: self.minimal_pdo_set,
            default_pdo_set: self.default_pdo_set,
            sync_rx: Some(sync_rx),
            cmd_tx: self.cmd_tx,
            cmd_rx: self.cmd_rx,
            _canopen: self._canopen,
            _mapping: self._mapping,
            _sync: std::marker::PhantomData,
            _mode: self._mode,
        }
    }
}

impl<C, M, S, Mode> Cia402DriverBuilder<C, M, S, Mode> {
    pub fn with_parameters(mut self, params: &'static [SdoAction<'static>]) -> Self {
        self.parameters = Some(params);
        self
    }

    pub fn with_default_parameters(mut self) -> Self {
        self.parameters = Some(TEST_PARAMS);
        self
    }
}

impl<C, M, S> Cia402DriverBuilder<C, M, S, Standalone> {
    /// Configure the Cia402Driver as slave device in a mechanically linked system
    pub fn as_slave_with_master(
        self,
        master: &Cia402Driver<AxisMaster>,
    ) -> Cia402DriverBuilder<C, M, S, AxisSlave> {
        Cia402DriverBuilder {
            identifier: self.identifier,
            canopen: self.canopen,
            parameters: self.parameters,
            minimal_pdo_set: self.minimal_pdo_set,
            default_pdo_set: self.default_pdo_set,
            sync_rx: self.sync_rx,
            cmd_tx: Some(master.get_cmd_tx_channel()),
            cmd_rx: Some(master.get_cmd_rx_channel()),
            _canopen: self._canopen,
            _mapping: self._mapping,
            _sync: self._sync,
            _mode: std::marker::PhantomData,
        }
    }

    /// Configure the Cia402Driver as master device in a mechanically linked system
    pub fn as_master(self) -> Cia402DriverBuilder<C, M, S, AxisMaster> {
        let (cmd_tx, cmd_rx) = tokio::sync::broadcast::channel(10);

        Cia402DriverBuilder {
            identifier: self.identifier,
            canopen: self.canopen,
            parameters: self.parameters,
            minimal_pdo_set: self.minimal_pdo_set,
            default_pdo_set: self.default_pdo_set,
            sync_rx: self.sync_rx,
            cmd_tx: Some(cmd_tx),
            cmd_rx: Some(cmd_rx),
            _canopen: self._canopen,
            _mapping: self._mapping,
            _sync: self._sync,
            _mode: std::marker::PhantomData,
        }
    }
}

// Only allowed when all requried configuration is passed
impl<Mode> Cia402DriverBuilder<HasCanOpen, HasMapping, HasSyncReceiver, Mode> {
    /// Build the Cia402Driver
    pub async fn build(self) -> Result<Cia402Driver<Mode>, InitialisationError> {
        let canopen = self.canopen.unwrap();
        let minimal_pdo_set = self.minimal_pdo_set.unwrap();
        let default_pdo_set = self.default_pdo_set.unwrap();
        let sync_rx = self.sync_rx.unwrap();
        let params = self.parameters.unwrap_or(&[]);
        let identifier = self.identifier;

        // Check if external command interface was provided
        let (cmd_tx, cmd_rx) = if let Some(cmd_tx) = self.cmd_tx {
            match self.cmd_rx {
                // Both parts of external channel found; use them
                Some(cmd_rx) => (cmd_tx, cmd_rx),
                // Missing half of the channel; error out
                None => {
                    return Err(InitialisationError::ExternalRXCommandChannelMissing(
                        identifier,
                    ));
                }
            }
        } else {
            // No external cmd channel provided; construct our own
            tokio::sync::broadcast::channel(1)
        };

        Cia402Driver::spawn_tasks(
            identifier,
            canopen,
            params,
            default_pdo_set,
            minimal_pdo_set,
            sync_rx,
            cmd_tx,
            cmd_rx,
        )
        .await
    }
}

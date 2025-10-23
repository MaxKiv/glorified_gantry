use oze_canopen::interface::CanOpenInterface;
use tokio::{sync::broadcast, time::Instant};

use crate::{
    comms::{
        pdo::mapping::{
            PDOSet, custom::CUSTOM_PDOS, minimal::MINIMAL_CYCLIC_SYNCHRONOUS_PDO_SET,
        },
        sdo::SdoAction,
    },
    driver::{Cia402Driver, startup::params::PARAMS},
    error::DriveError,
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
pub struct Cia402DriverBuilder<C, M, S> {
    node_id: u8,
    name: Option<String>,
    canopen: Option<CanOpenInterface>,
    parameters: Option<&'static [SdoAction<'static>]>,
    default_pdo_set: Option<&'static PDOSet>,
    minimal_pdo_set: Option<&'static PDOSet>,
    sync_rx: Option<broadcast::Receiver<Instant>>,
    _canopen: std::marker::PhantomData<C>,
    _mapping: std::marker::PhantomData<M>,
    _sync: std::marker::PhantomData<S>,
}

impl Cia402DriverBuilder<NoCanOpen, NoMapping, NoSyncReceiver> {
    /// Start building a new Cia402Driver
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
            name: None,
            canopen: None,
            parameters: None,
            default_pdo_set: None,
            minimal_pdo_set: None,
            sync_rx: None,
            _canopen: std::marker::PhantomData,
            _mapping: std::marker::PhantomData,
            _sync: std::marker::PhantomData,
        }
    }
}

// Required Cia402Driver configuration
impl<M, S> Cia402DriverBuilder<NoCanOpen, M, S> {
    /// Configure the Cia402Driver with a CANOpen interface
    pub fn with_canopen(self, iface: CanOpenInterface) -> Cia402DriverBuilder<HasCanOpen, M, S> {
        Cia402DriverBuilder {
            node_id: self.node_id,
            name: self.name,
            canopen: Some(iface),
            parameters: self.parameters,
            minimal_pdo_set: self.minimal_pdo_set,
            default_pdo_set: self.default_pdo_set,
            sync_rx: self.sync_rx,
            _canopen: std::marker::PhantomData,
            _mapping: std::marker::PhantomData,
            _sync: std::marker::PhantomData,
        }
    }
}

// Required Cia402Driver configuration
impl<C, S> Cia402DriverBuilder<C, NoMapping, S> {
    /// Configure the Cia402Driver with a T/RPDO mapping
    pub fn with_default_pdo_mappings(self) -> Cia402DriverBuilder<C, HasMapping, S> {
        let default_pdo_set = DEFAULT_PDO_SET;
        let minimal_pdo_set = MINIMAL_PDO_SET;

        self.with_pdo_mappings(default_pdo_set, minimal_pdo_set)
    }

    fn with_pdo_mappings(
        self,
        default_pdo_set: &'static PDOSet,
        minimal_pdo_set: &'static PDOSet,
    ) -> Cia402DriverBuilder<C, HasMapping, S> {
        if default_pdo_set.contains_default_rpdo() && minimal_pdo_set.contains_minimal_rpdo() {
            Cia402DriverBuilder {
                node_id: self.node_id,
                name: self.name,
                canopen: self.canopen,
                parameters: self.parameters,
                default_pdo_set: Some(default_pdo_set),
                minimal_pdo_set: Some(minimal_pdo_set),
                sync_rx: self.sync_rx,
                _canopen: std::marker::PhantomData,
                _mapping: std::marker::PhantomData,
                _sync: std::marker::PhantomData,
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
impl<C, M> Cia402DriverBuilder<C, M, NoSyncReceiver> {
    /// Configure the Cia402Driver with a T/RPDO mapping
    pub fn with_sync_receiver(
        self,
        sync_rx: broadcast::Receiver<Instant>,
    ) -> Cia402DriverBuilder<C, M, HasSyncReceiver> {
        Cia402DriverBuilder {
            node_id: self.node_id,
            name: self.name,
            canopen: self.canopen,
            parameters: self.parameters,
            minimal_pdo_set: self.minimal_pdo_set,
            default_pdo_set: self.default_pdo_set,
            sync_rx: Some(sync_rx),
            _canopen: std::marker::PhantomData,
            _mapping: std::marker::PhantomData,
            _sync: std::marker::PhantomData,
        }
    }
}

impl<C, M, S> Cia402DriverBuilder<C, M, S> {
    pub fn with_parameters(mut self, params: &'static [SdoAction<'static>]) -> Self {
        self.parameters = Some(params);
        self
    }

    pub fn with_default_parameters(mut self) -> Self {
        self.parameters = Some(PARAMS);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

// Only allowed when all requried configuration is passed
impl Cia402DriverBuilder<HasCanOpen, HasMapping, HasSyncReceiver> {
    /// Build the Cia402Driver
    pub async fn build(self) -> Result<Cia402Driver, DriveError> {
        let canopen = self.canopen.unwrap();
        let minimal_pdo_set = self.minimal_pdo_set.unwrap();
        let default_pdo_set = self.default_pdo_set.unwrap();
        let sync_rx = self.sync_rx.unwrap();
        let params = self.parameters.unwrap_or(&[]);
        let name = self.name.unwrap_or(format!("Motor #{}", self.node_id));

        Cia402Driver::spawn_tasks(
            self.node_id,
            name,
            canopen,
            params,
            default_pdo_set,
            minimal_pdo_set,
            sync_rx,
        )
        .await
    }
}

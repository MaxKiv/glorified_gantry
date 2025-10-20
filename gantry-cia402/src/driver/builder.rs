use oze_canopen::interface::CanOpenInterface;

use crate::{
    comms::{pdo::mapping::PdoMapping, sdo::SdoAction},
    driver::Cia402Driver,
    error::DriveError,
};

// Typestate structs
/// No CANOpen configured yet
pub struct NoCanOpen;
/// With valid CANOpen configuration
pub struct HasCanOpen;

/// No T/RPDO mapping configured yet
pub struct NoMapping;
/// With valid T/RPDO mapping configured yet
pub struct HasMapping;

/// Typestate Builder for the Cia402Driver
pub struct Cia402DriverBuilder<C, M> {
    node_id: u8,
    canopen: Option<CanOpenInterface>,
    parameters: Option<&'static [SdoAction<'static>]>,
    rpdo_mapping: Option<&'static [PdoMapping]>,
    tpdo_mapping: Option<&'static [PdoMapping]>,
    _canopen: std::marker::PhantomData<C>,
    _mapping: std::marker::PhantomData<M>,
}

impl Cia402DriverBuilder<NoCanOpen, NoMapping> {
    /// Start building a new Cia402Driver
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
            canopen: None,
            parameters: None,
            rpdo_mapping: None,
            tpdo_mapping: None,
            _canopen: std::marker::PhantomData,
            _mapping: std::marker::PhantomData,
        }
    }
}

// Required Cia402Driver configuration
impl<M> Cia402DriverBuilder<NoCanOpen, M> {
    /// Configure the Cia402Driver with a CANOpen interface
    pub fn with_canopen(self, iface: CanOpenInterface) -> Cia402DriverBuilder<HasCanOpen, M> {
        Cia402DriverBuilder {
            node_id: self.node_id,
            canopen: Some(iface),
            parameters: self.parameters,
            rpdo_mapping: self.rpdo_mapping,
            tpdo_mapping: self.tpdo_mapping,
            _canopen: std::marker::PhantomData,
            _mapping: std::marker::PhantomData,
        }
    }
}

// Required Cia402Driver configuration
impl<C> Cia402DriverBuilder<C, NoMapping> {
    /// Configure the Cia402Driver with a T/RPDO mapping
    pub fn with_pdo_mappings(
        self,
        rpdo: &'static [PdoMapping],
        tpdo: &'static [PdoMapping],
    ) -> Cia402DriverBuilder<C, HasMapping> {
        Cia402DriverBuilder {
            node_id: self.node_id,
            canopen: self.canopen,
            parameters: self.parameters,
            rpdo_mapping: Some(rpdo),
            tpdo_mapping: Some(tpdo),
            _canopen: std::marker::PhantomData,
            _mapping: std::marker::PhantomData,
        }
    }
}

impl<C, M> Cia402DriverBuilder<C, M> {
    pub fn with_parameters(mut self, params: &'static [SdoAction<'static>]) -> Self {
        self.parameters = Some(params);
        self
    }
}

// Only allowed when all requried configuration is passed
impl Cia402DriverBuilder<HasCanOpen, HasMapping> {
    /// Build the Cia402Driver
    pub async fn build(self) -> Result<Cia402Driver, DriveError> {
        let canopen = self.canopen.unwrap();
        let rpdo = self.rpdo_mapping.unwrap();
        let tpdo = self.tpdo_mapping.unwrap();
        let params = self.parameters.unwrap_or(&[]);

        Cia402Driver::spawn_tasks(self.node_id, canopen, params, rpdo, tpdo).await
    }
}

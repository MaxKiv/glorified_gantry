// TODO:8 Move this to its own crate

use crate::{
    canopen::pdo::mapping::OMSMotorPdoConfig,
    consts::pdo::{DEFAULT_MOTOR_PDOCFG, MotorPdoConfig, pp::DEFAULT_PP_PDOCFG},
};

/// Currently active PDO configuration for all 4 motors that make up the H-gantry
pub struct GantryOMSPdoConfig {
    pub inner: OMSMotorPdoConfig,
}

pub const DEFAULT_ACTIVE_GANTRY_PDOCFG: GantryOMSPdoConfig = GantryOMSPdoConfig {
    inner: DEFAULT_PP_PDOCFG,
};

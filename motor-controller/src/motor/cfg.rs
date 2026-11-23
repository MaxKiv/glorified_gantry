use crate::{
    canopen::sdo::action::SdoAction,
    cia402::Cia402Identifier,
    consts::{
        params::test::TEST_PARAMS,
        pdo::{NodePdoConfig, gantry::DEFAULT_GANTRY_PDOCFG},
    },
};

/// Configuration struct for a single CiA402 motor
#[derive(Clone)]
pub struct MotorConfig {
    /// Whats the masters CANopen node id
    pub id: Cia402Identifier,
    /// Required parameters for each motor of this axis
    pub default_parameters: &'static [SdoAction<'static>],
    /// Full PDO Configuration for this motor
    pub pdo: &'static NodePdoConfig,
    // /// Define how to map from SI units <-> Motor units
    // pub scaling: AxisScaling,
}

impl MotorConfig {
    pub const fn test(id: Cia402Identifier, pdo: &'static NodePdoConfig) -> Self {
        Self {
            id,
            default_parameters: TEST_PARAMS,
            pdo,
        }
    }
}

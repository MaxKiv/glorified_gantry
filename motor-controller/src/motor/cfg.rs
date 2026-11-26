use crate::{
    canopen::sdo::action::SdoAction,
    cia402::Cia402Identifier,
    consts::{TEST_LIMITS, params::test::TEST_PARAMS, pdo::MotorPdoConfig},
    motor::{limits::MotorLimits, scaling::MotorScaling},
};

/// Configuration struct for a single CiA402 motor
#[derive(Clone)]
pub struct MotorConfig {
    /// Whats the masters CANopen node id
    pub id: Cia402Identifier,
    /// Required parameters for each motor of this axis
    pub default_parameters: &'static [SdoAction<'static>],
    /// Full PDO Configuration for this motor
    pub pdo: &'static MotorPdoConfig,
    // /// Define how to map from SI units <-> Motor units
    pub scaling: MotorScaling,
    // /// Define how to map from SI units <-> Motor units
    pub limits: MotorLimits,
}

impl MotorConfig {
    pub const fn test(id: Cia402Identifier, pdo: &'static MotorPdoConfig) -> Self {
        Self {
            id,
            default_parameters: TEST_PARAMS,
            pdo,
            scaling: MotorScaling::test_setup(),
            limits: TEST_LIMITS,
        }
    }
}

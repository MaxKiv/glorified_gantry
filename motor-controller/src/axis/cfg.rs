use crate::{
    axis::scaling::AxisScaling, canopen::sdo::action::SdoAction, motor::cfg::MotorConfig,
    rt::engine::cfg::Axis,
};

/// Configuration struct for a single gantry axis
#[derive(Clone)]
pub struct AxisConfig {
    /// What axis is this config for
    pub axis: Axis,
    /// Whats the masters CANopen node id
    pub master: MotorConfig,
    /// The slave's node id, if there is one
    pub slave: Option<MotorConfig>,
    // /// Required parameters for each motor of this axis
    // /// TODO: refactor sdo stuff to not use vec
    // pub default_parameters: &'static [SdoAction<'static>],
    /// Define how to map from SI units <-> Motor units
    pub scaling: AxisScaling,
}

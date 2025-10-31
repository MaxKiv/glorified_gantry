use crate::axis::AxisConfig;

#[derive(Clone)]
pub struct GantryConfig {
    pub x: Option<AxisConfig>,
    pub y: Option<AxisConfig>,
    pub z: Option<AxisConfig>,
}

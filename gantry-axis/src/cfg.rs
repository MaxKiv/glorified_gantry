use crate::axis::AxisConfig;

pub struct GantryConfig {
    pub x: Option<AxisConfig>,
    pub y: Option<AxisConfig>,
    pub z: Option<AxisConfig>,
}

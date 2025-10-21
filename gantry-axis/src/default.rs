use crate::axis::{Axis, AxisConfig};

pub const DEFAULT_X_CONFIG: AxisConfig = AxisConfig {
    axis: Axis::X,
    master: 1,
    slave: None,
};

pub const DEFAULT_Y_CONFIG: AxisConfig = AxisConfig {
    axis: Axis::Y,
    master: 2,
    slave: None,
};

pub const DEFAULT_Z_CONFIG: AxisConfig = AxisConfig {
    axis: Axis::Z,
    master: 3,
    slave: None,
};

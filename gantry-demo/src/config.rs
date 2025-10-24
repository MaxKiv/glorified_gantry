use gantry_axis::axis::{Axis, AxisConfig};

pub const DEFAULT_X_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::X,
    master: 1,
    slave: None,
});

pub const DEFAULT_Y_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::Y,
    master: 2,
    slave: None,
});

pub const DEFAULT_Z_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::Z,
    master: 3,
    slave: None,
});

pub const TEST_X_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::X,
    master: 1,
    slave: None,
});

pub const TEST_Y_CONFIG: Option<AxisConfig> = None;

pub const TEST_Z_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::Z,
    master: 3,
    slave: None,
});

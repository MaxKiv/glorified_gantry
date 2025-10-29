use gantry_axis::{
    axis::{Axis, AxisConfig},
    setpoint::translator::scaling::DeviceScaling,
};
use gantry_cia402::driver::startup::params::{TEST_PARAMS, default::DEFAULT_PARAMS};

pub const DEFAULT_X_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::X,
    master: 1,
    slave: Some(2),
    params: DEFAULT_PARAMS,
    scaling: DeviceScaling::default_setup(),
});

// pub const DEFAULT_X_CONFIG: Option<AxisConfig> = None;

pub const DEFAULT_Y_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::Y,
    master: 3,
    slave: None,
    params: DEFAULT_PARAMS,
    scaling: DeviceScaling::default_setup(),
});

// pub const DEFAULT_Y_CONFIG: Option<AxisConfig> = None;

pub const DEFAULT_Z_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::Z,
    master: 4,
    slave: None,
    params: DEFAULT_PARAMS,
    scaling: DeviceScaling::default_setup(),
});

// pub const DEFAULT_Z_CONFIG: Option<AxisConfig> = None;

pub const TEST_X_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::X,
    master: 1,
    slave: None,
    params: TEST_PARAMS,
    scaling: DeviceScaling::test_setup(),
});

pub const TEST_Y_CONFIG: Option<AxisConfig> = None;

pub const TEST_Z_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::Z,
    master: 3,
    slave: None,
    params: TEST_PARAMS,
    scaling: DeviceScaling::test_setup(),
});

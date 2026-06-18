use gantry_axis::{
    axis::{Axis, AxisConfig},
    cfg::GantryConfig,
    setpoint::translator::scaling::DeviceScaling,
};
use gantry_cia402::driver::{
    identifier::{Cia402Identifier, CiaProfileNumber, MotorType},
    startup::params::{TEST_PARAMS, default::DEFAULT_PARAMS},
};

pub type DeviceName = &'static str;
pub const TEST_SETUP_PROFILE_NUMBER: CiaProfileNumber = CiaProfileNumber::Cia402;
pub const TEST_SETUP_MOTOR_TYPE: MotorType = MotorType::Stepper;
pub const TEST_SETUP_DEVICE_NAME: DeviceName = "PD4-E591L42-E-65-2";

pub const Z_ONLY_CONFIG: GantryConfig = GantryConfig {
    x: X_DISABLED,
    y: Y_DISABLED,
    z: DEFAULT_Z_CONFIG,
};

pub const YZ_CONFIG: GantryConfig = GantryConfig {
    x: X_DISABLED,
    y: DEFAULT_Y_CONFIG,
    z: DEFAULT_Z_CONFIG,
};

pub const DEFAULT_CONFIG: GantryConfig = GantryConfig {
    x: DEFAULT_X_CONFIG,
    y: DEFAULT_Y_CONFIG,
    z: DEFAULT_Z_CONFIG,
};

pub const DEFAULT_X_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::X,
    master: Cia402Identifier {
        node_id: 1,
        device_profile_number: TEST_SETUP_PROFILE_NUMBER,
        motor_type: TEST_SETUP_MOTOR_TYPE,
        device_name: TEST_SETUP_DEVICE_NAME,
    },
    slave: Some(Cia402Identifier {
        node_id: 2,
        device_profile_number: TEST_SETUP_PROFILE_NUMBER,
        motor_type: TEST_SETUP_MOTOR_TYPE,
        device_name: TEST_SETUP_DEVICE_NAME,
    }),
    params: DEFAULT_PARAMS,
    scaling: DeviceScaling::default_setup(),
});

pub const X_DISABLED: Option<AxisConfig> = None;
pub const Y_DISABLED: Option<AxisConfig> = None;

pub const DEFAULT_Y_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::Y,
    master: Cia402Identifier {
        node_id: 3,
        device_profile_number: TEST_SETUP_PROFILE_NUMBER,
        motor_type: TEST_SETUP_MOTOR_TYPE,
        device_name: TEST_SETUP_DEVICE_NAME,
    },
    slave: None,
    params: DEFAULT_PARAMS,
    scaling: DeviceScaling::default_setup(),
});

// pub const DEFAULT_Y_CONFIG: Option<AxisConfig> = None;

pub const DEFAULT_Z_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::Z,
    master: Cia402Identifier {
        node_id: 4,
        device_profile_number: TEST_SETUP_PROFILE_NUMBER,
        motor_type: TEST_SETUP_MOTOR_TYPE,
        device_name: TEST_SETUP_DEVICE_NAME,
    },
    slave: None,
    params: DEFAULT_PARAMS,
    scaling: DeviceScaling::default_setup(),
});

// pub const DEFAULT_Z_CONFIG: Option<AxisConfig> = None;

pub const TEST_CONFIG: GantryConfig = GantryConfig {
    x: TEST_X_CONFIG,
    y: TEST_Y_CONFIG,
    z: TEST_Z_CONFIG,
};

pub const TEST_X_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::X,
    master: Cia402Identifier {
        node_id: 1,
        device_profile_number: TEST_SETUP_PROFILE_NUMBER,
        motor_type: TEST_SETUP_MOTOR_TYPE,
        device_name: TEST_SETUP_DEVICE_NAME,
    },
    slave: None,
    params: TEST_PARAMS,
    scaling: DeviceScaling::test_setup(),
});

pub const TEST_Y_CONFIG: Option<AxisConfig> = None;

pub const TEST_Z_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::Z,
    master: Cia402Identifier {
        node_id: 3,
        device_profile_number: TEST_SETUP_PROFILE_NUMBER,
        motor_type: TEST_SETUP_MOTOR_TYPE,
        device_name: TEST_SETUP_DEVICE_NAME,
    },
    slave: None,
    params: TEST_PARAMS,
    scaling: DeviceScaling::test_setup(),
});

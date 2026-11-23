use crate::{
    axis::{cfg::AxisConfig, scaling::AxisScaling},
    canopen::frame::NodeId,
    cia402::{Cia402Identifier, CiaProfileNumber, MotorType},
    consts::pdo::{DEFAULT_NODE_PDOCFG, gantry::DEFAULT_GANTRY_PDOCFG},
    motor::cfg::MotorConfig,
    rt::engine::cfg::{Axis, GantryCfg},
};

pub type DeviceName = &'static str;
pub const TEST_SETUP_PROFILE_NUMBER: CiaProfileNumber = CiaProfileNumber::Cia402;
pub const TEST_SETUP_MOTOR_TYPE: MotorType = MotorType::Stepper;
pub const TEST_SETUP_DEVICE_NAME: DeviceName = "PD4-E591L42-E-65-2";

pub const DEMO_SETUP_PROFILE_NUMBER: CiaProfileNumber = CiaProfileNumber::Cia402;
pub const DEMO_SETUP_MOTOR_TYPE: MotorType = MotorType::Stepper;
pub const DEMO_SETUP_MOTOR_TYPE_Z: MotorType = MotorType::BLDC;
pub const DEMO_SETUP_DEVICE_NAME: DeviceName = "PD4-C6018L4204-E-08";
pub const DEMO_SETUP_DEVICE_NAME_Z: DeviceName = "PD4-CB59M024035-E-08";

pub const TEST_MOTOR_CFG_X: MotorConfig = MotorConfig::test(
    Cia402Identifier {
        node_id: NodeId(1),
        device_profile_number: TEST_SETUP_PROFILE_NUMBER,
        motor_type: TEST_SETUP_MOTOR_TYPE,
        device_name: TEST_SETUP_DEVICE_NAME,
    },
    &DEFAULT_GANTRY_PDOCFG.x_master,
);

pub const TEST_MOTOR_CFG_Z: MotorConfig = MotorConfig::test(
    Cia402Identifier {
        node_id: NodeId(3),
        device_profile_number: TEST_SETUP_PROFILE_NUMBER,
        motor_type: TEST_SETUP_MOTOR_TYPE,
        device_name: TEST_SETUP_DEVICE_NAME,
    },
    &DEFAULT_GANTRY_PDOCFG.z,
);

pub const TEST_AXIS_SCALING: AxisScaling = AxisScaling::test_setup();

pub const TEST_GANTRY_CFG: GantryCfg = GantryCfg {
    x: Some(AxisConfig {
        axis: Axis::X,
        master: TEST_MOTOR_CFG_X,
        slave: None,
        scaling: TEST_AXIS_SCALING,
    }),
    y: None,
    z: Some(AxisConfig {
        axis: Axis::Z,
        master: TEST_MOTOR_CFG_Z,
        slave: None,
        scaling: TEST_AXIS_SCALING,
    }),
};

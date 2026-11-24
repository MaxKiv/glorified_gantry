use crate::{
    canopen::pdo::mapping::OMSMotorPdoConfig,
    consts::pdo::{
        cp::DEFAULT_CP_PDOCFG, ct::DEFAULT_CT_PDOCFG, cv::DEFAULT_CV_PDOCFG, pp::DEFAULT_PP_PDOCFG,
        pt::DEFAULT_PT_PDOCFG, pv::DEFAULT_PV_PDOCFG,
    },
    oms::OperationMode,
};

pub mod cp;
pub mod ct;
pub mod cv;
pub mod gantry;
pub mod pp;
pub mod pt;
pub mod pv;

/// Full PDO Configuration of a single node
/// maps from every relevant OperationMode -> PdoConfig for that mode
pub struct MotorPdoConfig {
    profile_position: OMSMotorPdoConfig,
    profile_velocity: OMSMotorPdoConfig,
    profile_torque: OMSMotorPdoConfig,
    cyclic_position: OMSMotorPdoConfig,
    cyclic_velocity: OMSMotorPdoConfig,
    cyclic_torque: OMSMotorPdoConfig,
}

impl MotorPdoConfig {
    pub fn get_cfg_for_opmode(&'static self, mode: &OperationMode) -> &'static OMSMotorPdoConfig {
        match mode {
            OperationMode::CyclicSynchronousPosition => &self.cyclic_position,
            OperationMode::CyclicSynchronousVelocity => &self.cyclic_velocity,
            OperationMode::CyclicSynchronousTorque => &self.cyclic_torque,
            OperationMode::ProfileVelocity => &self.profile_velocity,
            OperationMode::ProfileTorque => &self.profile_torque,
            OperationMode::ProfilePosition => &self.profile_position,
            OperationMode::Velocity => &self.profile_velocity,
            OperationMode::InterpolatedPosition => &self.profile_position,
            OperationMode::Homing => &self.profile_position,
            _ => &self.profile_position,
        }
    }
}

/// Default configuration of a single node/motor
pub const DEFAULT_MOTOR_PDOCFG: MotorPdoConfig = MotorPdoConfig {
    profile_position: DEFAULT_PP_PDOCFG,
    profile_velocity: DEFAULT_PV_PDOCFG,
    profile_torque: DEFAULT_PT_PDOCFG,
    cyclic_position: DEFAULT_CP_PDOCFG,
    cyclic_velocity: DEFAULT_CV_PDOCFG,
    cyclic_torque: DEFAULT_CT_PDOCFG,
};

pub const TEST_MOTOR_PDOCFG: &MotorPdoConfig = &DEFAULT_MOTOR_PDOCFG;
pub const DEMO_MOTOR_PDOCFG: &MotorPdoConfig = &DEFAULT_MOTOR_PDOCFG;

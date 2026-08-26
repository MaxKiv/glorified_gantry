use crate::{
    canopen::pdo::mapping::OMSNodePdoConfig,
    consts::pdo::{
        cp::DEFAULT_CP_PDOCFG, ct::DEFAULT_CT_PDOCFG, cv::DEFAULT_CV_PDOCFG, pp::DEFAULT_PP_PDOCFG,
        pt::DEFAULT_PT_PDOCFG, pv::DEFAULT_PV_PDOCFG,
    },
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
pub struct NodePdoConfig {
    profile_position: OMSNodePdoConfig,
    profile_velocity: OMSNodePdoConfig,
    profile_torque: OMSNodePdoConfig,
    cyclic_position: OMSNodePdoConfig,
    cyclic_velocity: OMSNodePdoConfig,
    cyclic_torque: OMSNodePdoConfig,
}

/// Default configuration of a single node/motor
const DEFAULT_NODE_PDOCFG: NodePdoConfig = NodePdoConfig {
    profile_position: DEFAULT_PP_PDOCFG,
    profile_velocity: DEFAULT_PV_PDOCFG,
    profile_torque: DEFAULT_PT_PDOCFG,
    cyclic_position: DEFAULT_CP_PDOCFG,
    cyclic_velocity: DEFAULT_CV_PDOCFG,
    cyclic_torque: DEFAULT_CT_PDOCFG,
};

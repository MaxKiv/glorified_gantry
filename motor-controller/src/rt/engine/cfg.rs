use crate::{
    canopen::{frame::NodeId, pdo::mapping::OMSNodePdoConfig},
    consts::pdo::{
        gantry::{
            DEFAULT_ACTIVE_GANTRY_PDOCFG, DEFAULT_GANTRY_PDOCFG, HGantryActivePdoConfig,
            HGantryPdoConfig,
        },
        pp::DEFAULT_PP_PDOCFG,
    },
    oms::OperationMode,
};

// TODO:8. Move gantry specific stuff to its own crate
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum GantryMotorType {
    Xmaster,
    Xslave,
    Y,
    Z,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct GantryMotor {
    pub node_id: NodeId,
    pub kind: GantryMotorType,
}

pub struct MotorState {
    pub mode: OperationMode,
    pub pdo_cfg: OMSNodePdoConfig,
}

impl Default for MotorState {
    fn default() -> Self {
        MotorState {
            mode: OperationMode::Homing,
            pdo_cfg: DEFAULT_PP_PDOCFG,
        }
    }
}

/// Mutable/changable part of the RtEngine configuration
pub struct MutableRtEngineConfig {
    pub mode: OperationMode,
    pub current_pdo_cfg: HGantryActivePdoConfig,
}

pub const DEFAULT_MUT_RT_ENGINE_CFG: MutableRtEngineConfig = MutableRtEngineConfig {
    mode: OperationMode::Homing,
    current_pdo_cfg: DEFAULT_ACTIVE_GANTRY_PDOCFG,
};

/// Constant/unchangable part of the RtEngine configuration
pub struct ConstRtEngineConfig {
    pub gantry_oms_pdo_cfg: HGantryPdoConfig,
    pub nodes: &'static [GantryMotor],
}

pub const TEST_CONST_RT_ENGINE_CFG: ConstRtEngineConfig = ConstRtEngineConfig {
    gantry_oms_pdo_cfg: DEFAULT_GANTRY_PDOCFG,
    nodes: TEST_GANTRY_NODES,
};

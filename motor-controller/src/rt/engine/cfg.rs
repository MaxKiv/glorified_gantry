use crate::{
    canopen::frame::NodeId,
    consts::pdo::gantry::{
        DEFAULT_ACTIVE_GANTRY_PDOCFG, DEFAULT_GANTRY_PDOCFG, HGantryActivePdoConfig,
        HGantryNodeMap, HGantryPdoConfig, TEST_GANTRY_NODEMAP, TEST_GANTRY_NODES,
    },
    oms::OperationMode,
};

// TODO:8. Move gantry specific stuff to its own crate

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
pub struct ConstRtEngineConfig<const N: usize> {
    pub gantry_oms_pdo_cfg: HGantryPdoConfig,
    pub node_map: HGantryNodeMap,
    pub nodes: [NodeId; N],
}

pub const TEST_CONST_RT_ENGINE_CFG: ConstRtEngineConfig<2> = ConstRtEngineConfig {
    gantry_oms_pdo_cfg: DEFAULT_GANTRY_PDOCFG,
    node_map: TEST_GANTRY_NODEMAP,
    nodes: TEST_GANTRY_NODES,
};

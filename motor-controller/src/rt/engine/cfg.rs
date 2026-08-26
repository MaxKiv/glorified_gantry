use crate::{
    consts::pdo::gantry::{
        DEFAULT_ACTIVE_GANTRY_PDOCFG, DEFAULT_GANTRY_PDOCFG, HGantryActivePdoConfig,
        HGantryNodeMap, HGantryPdoConfig, TEST_GANTRY_NODEMAP,
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
pub struct ConstRtEngineConfig {
    pub gantry_oms_pdo_cfg: HGantryPdoConfig,
    pub node_map: HGantryNodeMap,
}

pub const CONST_RT_ENGINE_CFG: ConstRtEngineConfig = ConstRtEngineConfig {
    gantry_oms_pdo_cfg: DEFAULT_GANTRY_PDOCFG,
    node_map: TEST_GANTRY_NODEMAP,
};

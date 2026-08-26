// TODO:8 Move this to its own crate

use crate::{
    canopen::{frame::NodeId, pdo::mapping::OMSNodePdoConfig},
    consts::pdo::{DEFAULT_NODE_PDOCFG, NodePdoConfig, pp::DEFAULT_PP_PDOCFG},
    oms::OperationMode,
};

// TODO:7. Merge HGantryNodeMap -> AxisConfiguration?
pub struct HGantryNodeMap {
    pub x_master: NodeId,
    pub x_slave: NodeId,
    pub y: NodeId,
    pub z: NodeId,
}

pub const TEST_GANTRY_NODEMAP: HGantryNodeMap = HGantryNodeMap {
    x_master: NodeId(1),
    x_slave: NodeId(0),
    y: NodeId(3),
    z: NodeId(0),
};

pub const DEMO_GANTRY_NODEMAP: HGantryNodeMap = HGantryNodeMap {
    x_master: NodeId(1),
    x_slave: NodeId(2),
    y: NodeId(3),
    z: NodeId(4),
};

/// PDO configuration for every node/motor that makes up a H-gantry
pub struct HGantryPdoConfig {
    pub x1: NodePdoConfig,
    pub x2: NodePdoConfig,
    pub y: NodePdoConfig,
    pub z: NodePdoConfig,
}

pub const DEFAULT_GANTRY_PDOCFG: HGantryPdoConfig = HGantryPdoConfig {
    x1: DEFAULT_NODE_PDOCFG,
    x2: DEFAULT_NODE_PDOCFG,
    y: DEFAULT_NODE_PDOCFG,
    z: DEFAULT_NODE_PDOCFG,
};

/// Currently active PDO configuration for all 4 motors that make up the H-gantry
pub struct HGantryActivePdoConfig {
    pub x1: OMSNodePdoConfig,
    pub x2: OMSNodePdoConfig,
    pub y: OMSNodePdoConfig,
    pub z: OMSNodePdoConfig,
}

pub const DEFAULT_ACTIVE_GANTRY_PDOCFG: HGantryActivePdoConfig = HGantryActivePdoConfig {
    x1: DEFAULT_PP_PDOCFG,
    x2: DEFAULT_PP_PDOCFG,
    y: DEFAULT_PP_PDOCFG,
    z: DEFAULT_PP_PDOCFG,
};

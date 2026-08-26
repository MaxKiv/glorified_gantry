// TODO: Move this to its own crate

use crate::{
    canopen::{frame::NodeId, pdo::mapping::OMSNodePdoConfig},
    consts::pdo::{DEFAULT_NODE_PDOCFG, NodePdoConfig, pp::DEFAULT_PP_PDOCFG},
    oms::OperationMode,
};

pub struct HGantryNodeMap {
    x_master: NodeId,
    x_slave: NodeId,
    y: NodeId,
    z: NodeId,
}

/// TODO:6 validate
pub const TEST_GANTRY_NODEMAP: HGantryNodeMap = HGantryNodeMap {
    x_master: NodeId(0),
    x_slave: NodeId(1),
    y: NodeId(3),
    z: NodeId(4),
};

/// TODO:6 validate
pub const DEMO_GANTRY_NODEMAP: HGantryNodeMap = HGantryNodeMap {
    x_master: NodeId(1),
    x_slave: NodeId(2),
    y: NodeId(3),
    z: NodeId(4),
};

/// PDO configuration for every node/motor that makes up a H-gantry
pub struct HGantryPdoConfig {
    x1: NodePdoConfig,
    x2: NodePdoConfig,
    y: NodePdoConfig,
    z: NodePdoConfig,
}

pub const DEFAULT_GANTRY_PDOCFG: HGantryPdoConfig = HGantryPdoConfig {
    x1: DEFAULT_NODE_PDOCFG,
    x2: DEFAULT_NODE_PDOCFG,
    y: DEFAULT_NODE_PDOCFG,
    z: DEFAULT_NODE_PDOCFG,
};

/// Currently active PDO configuration for all 4 motors that make up the H-gantry
pub struct HGantryActivePdoConfig {
    mode: OperationMode,
    x1: OMSNodePdoConfig,
    x2: OMSNodePdoConfig,
    y: OMSNodePdoConfig,
    z: OMSNodePdoConfig,
}

pub const DEFAULT_ACTIVE_GANTRY_PDOCFG: HGantryActivePdoConfig = HGantryActivePdoConfig {
    mode: OperationMode::ProfilePosition,
    x1: DEFAULT_PP_PDOCFG,
    x2: DEFAULT_PP_PDOCFG,
    y: DEFAULT_PP_PDOCFG,
    z: DEFAULT_PP_PDOCFG,
};

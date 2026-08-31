// TODO:8 Move this to its own crate

use crate::{
    canopen::{frame::NodeId, pdo::mapping::OMSNodePdoConfig},
    consts::{
        MAX_NODE_ID,
        pdo::{DEFAULT_NODE_PDOCFG, NodePdoConfig, pp::DEFAULT_PP_PDOCFG},
    },
    rt::engine::cfg::{GantryMotor, GantryMotorType, MotorState},
};

/// PDO configuration for every node/motor that makes up a H-gantry
pub struct HGantryPdoConfig {
    pub x_master: NodePdoConfig,
    pub x_slave: NodePdoConfig,
    pub y: NodePdoConfig,
    pub z: NodePdoConfig,
}

pub const DEFAULT_GANTRY_PDOCFG: HGantryPdoConfig = HGantryPdoConfig {
    x_master: DEFAULT_NODE_PDOCFG,
    x_slave: DEFAULT_NODE_PDOCFG,
    y: DEFAULT_NODE_PDOCFG,
    z: DEFAULT_NODE_PDOCFG,
};

/// Currently active PDO configuration for all 4 motors that make up the H-gantry
pub struct HGantryActivePdoConfig {
    pub x_master: OMSNodePdoConfig,
    pub x_slave: OMSNodePdoConfig,
    pub y: OMSNodePdoConfig,
    pub z: OMSNodePdoConfig,
}

pub const DEFAULT_ACTIVE_GANTRY_PDOCFG: HGantryActivePdoConfig = HGantryActivePdoConfig {
    x_master: DEFAULT_PP_PDOCFG,
    x_slave: DEFAULT_PP_PDOCFG,
    y: DEFAULT_PP_PDOCFG,
    z: DEFAULT_PP_PDOCFG,
};

pub const TEST_MOTORS: [Option<GantryMotor>; MAX_NODE_ID] = [
    Some(GantryMotor {
        node_id: NodeId(1),
        kind: GantryMotorType::Y,
    }),
    None,
    Some(GantryMotor {
        node_id: NodeId(3),
        kind: GantryMotorType::Z,
    }),
    None,
];

// pub const DEMO_GANTRY_NODES: &[GantryMotor] = &[
//     GantryMotor {
//         node_id: NodeId::new(1),
//         kind: GantryMotorType::Xmaster,
//     },
//     GantryMotor {
//         node_id: NodeId::new(2),
//         kind: GantryMotorType::Xslave,
//     },
//     GantryMotor {
//         node_id: NodeId::new(3),
//         kind: GantryMotorType::Y,
//     },
//     GantryMotor {
//         node_id: NodeId::new(4),
//         kind: GantryMotorType::Z,
//     },
// ];
//

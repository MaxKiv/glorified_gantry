use crate::{
    axis::scaling::AxisScaling,
    canopen::{frame::NodeId, pdo::mapping::OMSNodePdoConfig},
    cia402::Cia402Identifier,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

pub struct GantryAxisCfg {
    pub x: Option<AxisConfig>,
    pub y: Option<AxisConfig>,
    pub z: Option<AxisConfig>,
}

impl GantryAxisCfg {
    /// Iterate over all configured motors: (axis config, which motor, node id)
    pub fn motors(&self) -> impl Iterator<Item = (Axis, &Cia402Identifier)> {
        [
            self.x.as_ref().map(|a| (a.axis, &a.master)),
            self.x
                .as_ref()
                .and_then(|a| a.slave.as_ref().map(|s| (a.axis, s))),
            self.y.as_ref().map(|a| (a.axis, &a.master)),
            self.y
                .as_ref()
                .and_then(|a| a.slave.as_ref().map(|s| (a.axis, s))),
            self.z.as_ref().map(|a| (a.axis, &a.master)),
            self.z
                .as_ref()
                .and_then(|a| a.slave.as_ref().map(|s| (a.axis, s))),
        ]
        .into_iter()
        .flatten()
    }
}

/// Configuration struct for a single gantry axis
#[derive(Clone)]
pub struct AxisConfig {
    /// What axis is this config for
    pub axis: Axis,
    /// Whats the masters CANopen node id
    pub master: Cia402Identifier,
    /// The slave's node id, if there is one
    pub slave: Option<Cia402Identifier>,
    /// Required parameters for each motor of this axis
    /// TODO: refactor sdo stuff to not use vec
    pub default_parameters: &'static [SdoAction<'static>],
    /// Define how to map from SI units <-> Motor units
    pub scaling: AxisScaling,
}

/// Constant/unchangable part of the RtEngine configuration
pub struct ConstRtEngineConfig {
    pub gantry_oms_pdo_cfg: HGantryPdoConfig,
    pub axis_cfg: GantryAxisCfg,
}

pub const TEST_CONST_RT_ENGINE_CFG: ConstRtEngineConfig = ConstRtEngineConfig {
    gantry_oms_pdo_cfg: DEFAULT_GANTRY_PDOCFG,
    axis_cfg: TEST_AXIS_CFG,
};

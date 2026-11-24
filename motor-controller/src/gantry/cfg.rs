use crate::{
    axis::{Axis, cfg::AxisConfig},
    cia402::Cia402Identifier,
};

/// Configuration struct for a full gantry
#[derive(Clone)]
pub struct GantryCfg {
    pub x: Option<AxisConfig>,
    pub y: Option<AxisConfig>,
    pub z: Option<AxisConfig>,
}

impl GantryCfg {
    /// Iterate over all configured motors: (axis config, which motor, node id)
    pub fn motors(&self) -> impl Iterator<Item = (Axis, &Cia402Identifier)> {
        [
            self.x.as_ref().map(|a| (a.axis, &a.master.id)),
            self.x
                .as_ref()
                .and_then(|a| a.slave.as_ref().map(|s| (a.axis, &s.id))),
            self.y.as_ref().map(|a| (a.axis, &a.master.id)),
            self.y
                .as_ref()
                .and_then(|a| a.slave.as_ref().map(|s| (a.axis, &s.id))),
            self.z.as_ref().map(|a| (a.axis, &a.master.id)),
            self.z
                .as_ref()
                .and_then(|a| a.slave.as_ref().map(|s| (a.axis, &s.id))),
        ]
        .into_iter()
        .flatten()
    }
}

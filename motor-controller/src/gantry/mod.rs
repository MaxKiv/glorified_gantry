use tracing::error;

use crate::{
    axis::GantryAxis,
    gantry::{cfg::GantryCfg, setpoint::GantrySetpoint},
    motor::ctx::TickCtx,
};

pub mod cfg;
pub mod setpoint;

pub struct Gantry {
    x: Option<GantryAxis>,
    y: Option<GantryAxis>,
    z: Option<GantryAxis>,
}

impl Gantry {
    pub fn with_cfg(gantry_cfg: GantryCfg) -> Self {
        // Start all axi
        let x = gantry_cfg.x.map(|c| GantryAxis::start_parametrised(c));
        let y = gantry_cfg.y.map(|c| GantryAxis::start_parametrised(c));
        let z = gantry_cfg.z.map(|c| GantryAxis::start_parametrised(c));

        Gantry { x, y, z }
    }

    /// Get a shared ref to all axi in this gantry
    pub fn get_axi_ref(&self) -> impl Iterator<Item = &GantryAxis> {
        [self.x.as_ref(), self.y.as_ref(), self.z.as_ref()]
            .into_iter()
            .flatten()
    }

    /// Get a mut ref to all axi in this gantry
    pub fn get_axi_mut(&mut self) -> impl Iterator<Item = &mut GantryAxis> {
        [self.x.as_mut(), self.y.as_mut(), self.z.as_mut()]
            .into_iter()
            .flatten()
    }

    /// Performs an Emergency Stop of the gantry
    pub fn do_e_stop(&mut self) {
        error!(system = "Gantry", "ESTOP TRIGGERED",);

        for axis in self.get_axi_mut() {
            axis.do_e_stop();
        }

        self.new_gantry_setpoint(GantrySetpoint::estop_setpoint());
    }

    pub fn tick(&mut self, ctx: &mut TickCtx) {
        for axis in self.get_axi_mut() {
            axis.tick(ctx);
        }
    }

    pub fn all_cycle_tpdo_received(&self) -> bool {
        for axis in self.get_axi_ref() {
            if !axis.all_cycle_tpdo_received() {
                return false;
            }
        }

        true
    }

    /// Push a newly received setpoint to each of the gantry axis
    pub fn new_gantry_setpoint(&mut self, setpoint: GantrySetpoint) {
        if let Some(axis) = &mut self.x
            && let Some(setpoint) = setpoint.x
        {
            axis.new_axis_setpoint(setpoint)
        }

        if let Some(axis) = &mut self.y
            && let Some(setpoint) = setpoint.y
        {
            axis.new_axis_setpoint(setpoint)
        }

        if let Some(axis) = &mut self.z
            && let Some(setpoint) = setpoint.z
        {
            axis.new_axis_setpoint(setpoint)
        }
    }
}

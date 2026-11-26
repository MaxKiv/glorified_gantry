use tracing::{debug, info};
use uom::si::f64::Length;

use crate::{
    axis::{cfg::AxisConfig, feedback::AxisFeedback, scaling::AxisScaling},
    canopen::frame::{CanOpenFrame, NodeId},
    motor::{Cia402Motor, MotorRpdoSet, ctx::TickCtx},
    oms::setpoint::Setpoint,
};

pub mod cfg;
pub mod error;
pub mod feedback;
pub mod scaling;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    pub const COUNT: usize = std::mem::variant_count::<Axis>();
}

pub struct GantryAxisRpdo {
    pub master: MotorRpdoSet,
    pub slave: Option<MotorRpdoSet>,
}

pub struct GantryAxis {
    pub axis: Axis,
    pub master: Cia402Motor,
    pub slave: Option<Cia402Motor>,
    pub scaling: AxisScaling,
}

impl GantryAxis {
    pub fn start_parametrised(cfg: AxisConfig) -> Self {
        // Start master
        let axis = cfg.axis;
        let scaling = cfg.scaling;
        let master = Cia402Motor::start_parametrised(cfg.master);
        let slave = cfg.slave.map(|s| Cia402Motor::start_parametrised(s));

        Self {
            axis,
            master,
            slave,
            scaling,
        }
    }

    /// Get mutable ref to motors that make up this axis
    /// NOTE: always yields &[master, slave] in order
    fn get_axis_motors_mut(&mut self) -> impl Iterator<Item = &mut Cia402Motor> {
        [Some(&mut self.master), self.slave.as_mut()]
            .into_iter()
            .flatten()
    }

    /// Get immutable ref to motors that make up this axis
    /// NOTE: always yields &[master, slave] in order
    fn get_axis_motors_ref(&self) -> impl Iterator<Item = &Cia402Motor> {
        [Some(&self.master), self.slave.as_ref()]
            .into_iter()
            .flatten()
    }

    /// Push a new setpoint to the axis master drive, and slave if any
    pub fn new_axis_setpoint(&mut self, setpoint: Setpoint) {
        // TODO: scale setpoint using AxisScaling

        if let Some(slave) = self.slave.as_mut() {
            self.master.new_motor_setpoint(setpoint.clone());
            slave.new_motor_setpoint(setpoint);
        } else {
            // Avoid setpoint clone
            self.master.new_motor_setpoint(setpoint);
        }
    }

    /// Schedules non-blocking (== non I/O) work for all motors in this axis
    pub fn tick(&mut self, ctx: &mut TickCtx) {
        debug!(system = "Axis", "tick()",);
        for motor in self.get_axis_motors_mut() {
            motor.tick(ctx);
        }
    }

    pub fn on_sync_feedback(&mut self) {
        info!(system = "Axis", "on_sync_feedback");
        for motor in self.get_axis_motors_mut() {
            motor.on_sync_feedback();
        }
    }

    // Returns skew distance between master & slave motors for this axis
    // Skew is only calculated for axi with both master & slave
    // Single motor axi return None
    pub fn skew(&self) -> Option<Length> {
        self.slave.as_ref().map(|s| {
            let master_pos = self.master.motor_state.pos;
            let slave_pos = s.motor_state.pos;

            if master_pos > slave_pos {
                master_pos - slave_pos
            } else {
                slave_pos - master_pos
            }
        })
    }

    pub fn managed_node_ids(&self) -> impl Iterator<Item = NodeId> {
        [
            Some(self.master.id.node_id),
            self.slave.as_ref().map(|s| s.id.node_id),
        ]
        .into_iter()
        .flatten()
    }

    pub fn process_canopen_msg(&mut self, msg: CanOpenFrame) {
        tracing::trace!(system = "Axis", "process_canopen_msg({:?})", msg);
        let Some(msg_id) = msg.node_id else {
            tracing::error!(
                system = "Axis",
                "GantryAxis::process_canopen_msg called msg that contains no node_id: {:?}",
                msg
            );
            assert!(
                false,
                "GantryAxis::process_canopen_msg called msg that contains no node_id: {:?}",
                msg
            );
            return;
        };

        for motor in self.get_axis_motors_mut() {
            if motor.id.node_id == msg_id {
                motor.process_canopen_msg(msg);
                return;
            }
        }
    }

    // Are all current cycle RPDO received for this axis
    pub fn all_cycle_tpdo_received(&self) -> bool {
        for motor in self.get_axis_motors_ref() {
            if !motor.all_cycle_tpdo_received() {
                return false;
            }
        }

        true
    }

    pub fn get_rpdos(&self) -> GantryAxisRpdo {
        GantryAxisRpdo {
            master: self.master.get_rpdo(),
            slave: self.slave.as_ref().map(|s| s.get_rpdo()),
        }
    }

    pub fn do_e_stop(&mut self) {
        tracing::warn!(system = "Axis", "AXIS ESTOP TRIGGERED");
        for motor in self.get_axis_motors_mut() {
            motor.do_e_stop();
        }
    }

    pub fn get_axis_feedback(&self) -> AxisFeedback {
        AxisFeedback {
            master: self.master.get_feedback(),
            slave: self.slave.as_ref().map(|s| s.get_feedback()),
        }
    }
}

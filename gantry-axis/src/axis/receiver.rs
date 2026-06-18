use gantry_cia402::driver::{event::MotorEvent, identifier::Cia402Identifier};
use oze_canopen::canopen::NodeId;
use tokio::sync::broadcast;

use crate::axis::Axis;

#[derive(Debug)]
pub struct AxisEventReceiver {
    pub axis: Axis,
    pub master_id: Cia402Identifier,
    pub master: broadcast::Receiver<MotorEvent>,
    pub slave_id: Option<Cia402Identifier>,
    pub slave: Option<broadcast::Receiver<MotorEvent>>,
}

impl AxisEventReceiver {
    pub fn new(
        axis: Axis,
        master_id: Cia402Identifier,
        master: broadcast::Receiver<MotorEvent>,
        slave_id: Option<Cia402Identifier>,
        slave: Option<broadcast::Receiver<MotorEvent>>,
    ) -> Self {
        Self {
            axis,
            master,
            slave,
            master_id,
            slave_id,
        }
    }
}

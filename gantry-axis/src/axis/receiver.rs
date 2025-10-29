use gantry_cia402::driver::event::MotorEvent;
use oze_canopen::canopen::NodeId;
use tokio::sync::broadcast;

use crate::axis::Axis;

pub struct AxisEventReceiver {
    pub axis: Axis,
    pub master_id: NodeId,
    pub master: broadcast::Receiver<MotorEvent>,
    pub slave_id: Option<NodeId>,
    pub slave: Option<broadcast::Receiver<MotorEvent>>,
}

impl AxisEventReceiver {
    pub fn new(
        axis: Axis,
        master_id: NodeId,
        master: broadcast::Receiver<MotorEvent>,
        slave_id: Option<NodeId>,
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

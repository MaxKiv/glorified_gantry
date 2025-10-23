use gantry_cia402::driver::event::MotorEvent;
use tokio::sync::broadcast;

use crate::axis::Axis;

pub struct AxisEventReceiver {
    pub axis: Axis,
    pub master: broadcast::Receiver<MotorEvent>,
    pub slave: Option<broadcast::Receiver<MotorEvent>>,
}

impl AxisEventReceiver {
    pub fn new(
        axis: Axis,
        master: broadcast::Receiver<MotorEvent>,
        slave: Option<broadcast::Receiver<MotorEvent>>,
    ) -> Self {
        Self {
            axis,
            master,
            slave,
        }
    }
}

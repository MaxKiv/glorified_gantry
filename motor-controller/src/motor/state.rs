use uom::si::{
    f64::{Length, Torque, Velocity},
    length::millimeter,
    torque::newton_meter,
    velocity::millimeter_per_second,
};

use crate::rt::MotorFeedback;

pub struct MotorState {
    pub pos: Length,
    pub vel: Velocity,
    pub torque: Torque,
}

impl MotorState {
    pub fn new() -> Self {
        Self {
            pos: Length::new::<millimeter>(0.0),
            vel: Velocity::new::<millimeter_per_second>(0.0),
            torque: Torque::new::<newton_meter>(0.0),
        }
    }

    // TODO: Improve this by adding motor scaling
    // Consider removing MotorFeedback altogether, update motorState directly with some
    // "motorscaling"
    pub fn update_from_motor_feedback(&mut self, feedback: &MotorFeedback) {
        self.pos = Length::new::<millimeter>(feedback.pos as f64);
        self.vel = Velocity::new::<millimeter_per_second>(feedback.vel as f64);
        self.torque = Torque::new::<newton_meter>(feedback.torque as f64);
    }
}

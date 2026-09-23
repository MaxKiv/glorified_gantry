use uom::si::{
    f64::{Length, Torque, Velocity},
    length::millimeter,
    torque::newton_meter,
    velocity::millimeter_per_second,
};

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
}

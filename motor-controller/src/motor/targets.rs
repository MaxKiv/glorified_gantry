// Collection of latest profile targets/setpoints per relevant mode
pub struct Cia402MotorTargets {
    pub target_position: i32,
    pub target_velocity: i32,
    pub target_torque: i16,
}

impl Cia402MotorTargets {
    pub fn new() -> Self {
        Self {
            target_position: 0,
            target_velocity: 0,
            target_torque: 0,
        }
    }
}

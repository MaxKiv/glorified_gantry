use uom::si::{
    f64::{Length, Torque, Velocity},
    length::millimeter,
    torque::newton_meter,
    velocity::millimeter_per_second,
};

use crate::{
    gantry::skew::GantrySkew, motor::state::MotorState, oms::OperationMode,
    rt::timekeeper::CycleTiming, sw::StatusWord,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MotorFeedback {
    pub sw: StatusWord,
    pub pos: Length,
    pub vel: Velocity,
    pub torque: Torque,
    pub opmode: OperationMode,
}

impl MotorFeedback {
    pub fn new() -> Self {
        Self {
            sw: StatusWord::default(),
            pos: Length::new::<millimeter>(0.0),
            vel: Velocity::new::<millimeter_per_second>(0.0),
            torque: Torque::new::<newton_meter>(0.0),
            opmode: OperationMode::default(),
        }
    }

    pub fn from_motor_state(state: &MotorState) -> MotorFeedback {
        MotorFeedback {
            sw: state.sw,
            pos: state.pos,
            vel: state.vel,
            torque: state.torque,
            opmode: state.opmode,
        }
    }
}

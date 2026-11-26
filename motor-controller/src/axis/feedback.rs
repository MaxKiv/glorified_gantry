use uom::si::f64::Length;

use crate::motor::feedback::MotorFeedback;

#[derive(Debug, Clone, PartialEq)]
pub struct AxisFeedback {
    pub master: MotorFeedback,
    pub slave: Option<MotorFeedback>,
}

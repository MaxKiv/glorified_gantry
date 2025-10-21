use uom::si::f64::*;
use uom::si::{length::millimeter, torque::newton_meter, velocity::meter_per_second};

#[derive(Debug, Clone)]
pub enum Setpoint {
    RelativePosition(PositionSetpoint),
    AbsolutePosition(PositionSetpoint),
    Velocity(VelocitySetpoint),
    Torque(TorqueSetpoint),
}

#[derive(Debug, Clone)]
pub struct PositionSetpoint {
    target: millimeter,
    velocity: meter_per_second,
}

#[derive(Debug, Clone)]
pub struct VelocitySetpoint {
    target: meter_per_second,
}

#[derive(Debug, Clone)]
pub struct TorqueSetpoint {
    target: newton_meter,
}

use uom::si::f64::*;

#[derive(Debug, Clone)]
pub enum AxisSetpoint {
    RelativePosition(PositionSetpoint),
    AbsolutePosition(PositionSetpoint),
    Velocity(VelocitySetpoint),
    Torque(TorqueSetpoint),
}

#[derive(Debug, Clone)]
pub struct PositionSetpoint {
    pub target: Length,
    pub velocity: Velocity,
}

#[derive(Debug, Clone)]
pub struct VelocitySetpoint {
    pub target: Velocity,
}

#[derive(Debug, Clone)]
pub struct TorqueSetpoint {
    pub target: Torque,
}

use crate::oms::{torque::TorqueSetpoint, velocity::VelocitySetpoint};

pub enum DefaultMotorSetpoint {
    AbsolutePosition(PositionSetpoint),
    RelativePosition(PositionSetpoint),
    Velocity(VelocitySetpoint),
    Torque(TorqueSetpoint),
}

pub enum CyclicMotorSetpoint {
    RelativePosition(PositionSetpoint),
    Velocity(VelocitySetpoint),
    Torque(TorqueSetpoint),
}

pub struct PositionSetpoint {
    target: i32,
}

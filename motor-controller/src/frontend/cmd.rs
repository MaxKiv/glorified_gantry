use crate::gantry::setpoint::GantrySetpoint;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum GantryCommand {
    /// Motors are Responsible for interpolation
    Setpoint(GantrySetpoint),
    /// You are Responsible for interpolation
    CyclicSetpoint(GantrySetpoint),
    /// Home motors
    Home,
    /// De Energise motors
    #[default]
    Idle,
    // ResetFaults, ?
}

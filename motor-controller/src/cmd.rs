use tokio::sync::oneshot;

use crate::{
    cia402::Cia402State,
    setpoint::{CyclicMotorSetpoint, DefaultMotorSetpoint},
};

pub struct MotorCommandChannel {
    cmd: MotorCommand,
    tx: oneshot::Sender<()>,
}

pub enum MotorCommand {
    Halt,
    Disable,
    Enable,
    Home,
    Cia402TransitionTo(Cia402State),
    Move(DefaultMotorSetpoint),
    EnterCyclicMode(CyclicMode),
    ExitCyclic(DefaultMode),
    MoveCyclic(CyclicMotorSetpoint),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DefaultMode {
    Homing,
    ProfilePosition,
    ProfileVelocity,
    ProfileTorque,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CyclicMode {
    CyclicPosition,
    CyclicVelocity,
    CyclicTorque,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OperationMode {
    Default(DefaultMode),
    Cyclic(CyclicMode),
}

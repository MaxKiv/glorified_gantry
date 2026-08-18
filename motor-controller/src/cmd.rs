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

pub enum DefaultMode {
    Homing,
    ProfilePosition,
    ProfileVelocity,
    ProfileTorque,
}

pub enum CyclicMode {
    CyclicPosition,
    CyclicVelocity,
    CyclicTorque,
}
pub enum OperationMode {
    Default(DefaultMode),
    Cyclic(CyclicMode),
}

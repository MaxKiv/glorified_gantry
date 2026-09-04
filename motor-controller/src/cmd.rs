use tokio::sync::oneshot;

use crate::{
    cia402::Cia402State,
    oms::OperationMode,
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
    EnterCyclicMode(OperationMode), // TODO: make EnterCyclicMode(PP) irrepresentable
    ExitCyclic(OperationMode),
    MoveCyclic(CyclicMotorSetpoint),
}

// #[derive(Debug, PartialEq, Clone, Copy)]
// #[repr(C)]
// pub enum OperationMode {
//     Homing = 0,
//     ProfilePosition = 1,
//     ProfileVelocity = 2,
//     ProfileTorque = 3,
//     CyclicPosition = 4,
//     CyclicVelocity = 5,
//     CyclicTorque = 6,
// }

// impl OperationMode {
//     pub const COUNT: usize = std::mem::variant_count::<OperationMode>();
//
//     pub fn is_cyclic(&self) -> bool {
//         match self {
//             OperationMode::CyclicPosition
//             | OperationMode::CyclicVelocity
//             | OperationMode::CyclicTorque => true,
//             _ => false,
//         }
//     }
// }

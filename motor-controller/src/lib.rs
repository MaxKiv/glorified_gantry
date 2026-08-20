// #![feature(core_io)]

pub mod cia402;
pub mod cmd;
pub mod consts;
pub mod cw;
pub mod event;
pub mod handshake;
pub mod oms;
pub mod rt;
pub mod setpoint;
pub mod sw;

use anyhow::bail;
use tokio::sync::{mpsc, watch};

use crate::{
    cia402::Cia402Manager, cmd::MotorCommand, handshake::HandshakeManager, rt::RtSetpoint,
};

pub struct MotorController {
    rx: mpsc::Receiver<MotorCommand>,
    handshake_manager: HandshakeManager,
    cia402_manager: Cia402Manager,
    tx: watch::Sender<RtSetpoint>,
}

impl MotorController {
    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            if let Some(cmd) = self.rx.recv().await {
                match cmd {
                    MotorCommand::Halt => todo!(),
                    MotorCommand::Disable => todo!(),
                    MotorCommand::Enable => todo!(),
                    MotorCommand::Home => todo!(),
                    MotorCommand::Cia402TransitionTo(s) => {
                        todo!()
                        // if let Err(err) = self.cia402_manager.try_transition_to(s) {}
                    }
                    MotorCommand::Move(default_motor_setpoint) => todo!(),
                    MotorCommand::EnterCyclicMode(cyclic_mode) => todo!(),
                    MotorCommand::ExitCyclic(default_mode) => todo!(),
                    MotorCommand::MoveCyclic(_) => todo!(),
                }
            } else {
                bail!("MotorController: command rx channel closed")
            }
        }
    }
}

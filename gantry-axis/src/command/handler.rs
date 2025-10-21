use tokio::{sync::mpsc, task::JoinHandle};
use tracing::*;

use crate::{
    axis::setpoint::AxisSetpoint,
    command::GantryCommand,
    setpoint::translator::{SetpointTranslator, scaling::DeviceScaling},
    spawn_logged,
};

pub struct CommandHandlerHandle {
    handle: JoinHandle<()>,
    cmd_tx: mpsc::Sender<GantryCommand>,
}

impl CommandHandlerHandle {
    pub fn get_cmd_tx(&self) -> mpsc::Sender<GantryCommand> {
        self.cmd_tx.clone()
    }
}

pub struct CommandHandler {}

impl CommandHandler {
    pub fn init() -> CommandHandlerHandle {
        info!("Initialising Setpoint Translator");
        let translator = SetpointTranslator::new(DeviceScaling::default());

        let (cmd_tx, cmd_rx) = mpsc::channel(10);

        let handle = spawn_logged("CMD", async move {
            CommandHandler::handle_commands(cmd_rx, translator).await
        });

        CommandHandlerHandle { handle, cmd_tx }
    }

    pub async fn handle_commands(
        mut cmd_rx: mpsc::Receiver<GantryCommand>,
        translator: SetpointTranslator,
    ) -> anyhow::Result<()> {
        loop {
            if let Some(cmd) = cmd_rx.recv().await {
                let motor_cmd = match cmd {
                    GantryCommand::SetAbsolutePosition { setpoint } => {
                        translator.to_motor_cmd(AxisSetpoint::AbsolutePosition(setpoint))
                    }
                    GantryCommand::SetRelativePosition { setpoint } => {
                        translator.to_motor_cmd(AxisSetpoint::RelativePosition(setpoint))
                    }
                    GantryCommand::SetVelocity { setpoint } => {
                        translator.to_motor_cmd(AxisSetpoint::Velocity(setpoint))
                    }
                    GantryCommand::SetTorque { setpoint } => {
                        translator.to_motor_cmd(AxisSetpoint::Torque(setpoint))
                    }
                };
            }
        }
    }
}

use gantry_cia402::driver::{command::MotorCommand, oms::setpoint::Setpoint};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::*;

use crate::{
    axis::AxisMotors,
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

pub struct CommandHandler;

impl CommandHandler {
    pub fn init(
        x_axis: AxisMotors,
        y_axis: AxisMotors,
        z_axis: AxisMotors,
    ) -> CommandHandlerHandle {
        info!("Initialising Setpoint Translator");
        let translator = SetpointTranslator::new(DeviceScaling::default());

        let (cmd_tx, cmd_rx) = mpsc::channel(10);

        let handle = spawn_logged("CMD", async move {
            CommandHandler::handle_commands(cmd_rx, translator, x_axis, y_axis, z_axis).await
        });

        CommandHandlerHandle { handle, cmd_tx }
    }

    pub async fn handle_commands(
        mut cmd_rx: mpsc::Receiver<GantryCommand>,
        translator: SetpointTranslator,
        x_axis: AxisMotors,
        y_axis: AxisMotors,
        z_axis: AxisMotors,
    ) -> anyhow::Result<()> {
        let mut cmd_x = None;
        let mut cmd_y = None;
        let mut cmd_z = None;

        loop {
            if let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    GantryCommand::Setpoint { x, y, z } => {
                        // Translate setpoints to motor units
                        cmd_x = x.map(|setpoint| translator.to_motor_cmd(setpoint));
                        cmd_y = y.map(|setpoint| translator.to_motor_cmd(setpoint));
                        cmd_z = z.map(|setpoint| translator.to_motor_cmd(setpoint));

                        // Send translated setpoint out to Axis motors
                        cmd_x.as_ref().map(|cmd| x_axis.send_command_to_motors(cmd));
                        cmd_y.as_ref().map(|cmd| y_axis.send_command_to_motors(cmd));
                        cmd_z.as_ref().map(|cmd| z_axis.send_command_to_motors(cmd));
                    }
                    GantryCommand::Home => {
                        let cmd = MotorCommand::Home;
                        x_axis.send_command_to_motors(&cmd);
                        y_axis.send_command_to_motors(&cmd);
                        z_axis.send_command_to_motors(&cmd);
                    }
                };
            }
        }
    }
}

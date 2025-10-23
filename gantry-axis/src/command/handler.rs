use gantry_cia402::driver::command::MotorCommand;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::*;

use crate::{
    axis::AxisMotors, command::GantryCommand, setpoint::translator::SetpointTranslator,
    spawn_logged,
};

pub struct CommandHandle {
    handle: JoinHandle<()>,
    pub cmd_tx: mpsc::Sender<GantryCommand>,
}

pub struct CommandHandler;

impl CommandHandler {
    pub fn init(
        x_motors: Option<AxisMotors>,
        y_motors: Option<AxisMotors>,
        z_motors: Option<AxisMotors>,
        translator: SetpointTranslator,
    ) -> CommandHandle {
        let (cmd_tx, cmd_rx) = mpsc::channel(10);

        let handle = spawn_logged("CMD", async move {
            CommandHandler::handle_commands(cmd_rx, translator, x_motors, y_motors, z_motors).await
        });

        CommandHandle { handle, cmd_tx }
    }

    pub async fn handle_commands(
        mut cmd_rx: mpsc::Receiver<GantryCommand>,
        translator: SetpointTranslator,
        x_axis: Option<AxisMotors>,
        y_axis: Option<AxisMotors>,
        z_axis: Option<AxisMotors>,
    ) -> anyhow::Result<()> {
        let mut cmd_x;
        let mut cmd_y;
        let mut cmd_z;

        loop {
            if let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    GantryCommand::Setpoint { x, y, z } => {
                        // Translate setpoints to motor units
                        cmd_x = x.map(|setpoint| translator.to_motor_cmd(setpoint));
                        cmd_y = y.map(|setpoint| translator.to_motor_cmd(setpoint));
                        cmd_z = z.map(|setpoint| translator.to_motor_cmd(setpoint));

                        // Send translated setpoint out to Axis motors
                        cmd_x
                            .as_ref()
                            .map(|cmd| x_axis.as_ref().map(|x| x.send_command_to_motors(cmd)));
                        cmd_y
                            .as_ref()
                            .map(|cmd| y_axis.as_ref().map(|y| y.send_command_to_motors(cmd)));
                        cmd_z
                            .as_ref()
                            .map(|cmd| z_axis.as_ref().map(|z| z.send_command_to_motors(cmd)));
                    }
                    GantryCommand::Home => {
                        info!("Gantry is Homing");

                        let cmd = MotorCommand::Enable;
                        // Send Enable command to each axis's motors
                        if let Some(x) = x_axis.as_ref() {
                            x.send_command_to_motors(&cmd)
                        }
                        if let Some(y) = y_axis.as_ref() {
                            y.send_command_to_motors(&cmd)
                        }
                        if let Some(z) = z_axis.as_ref() {
                            z.send_command_to_motors(&cmd)
                        }

                        let cmd = MotorCommand::Home;
                        // Send Home command to each axis's motors
                        if let Some(x) = x_axis.as_ref() {
                            x.send_command_to_motors(&cmd)
                        }
                        if let Some(y) = y_axis.as_ref() {
                            y.send_command_to_motors(&cmd)
                        }
                        if let Some(z) = z_axis.as_ref() {
                            z.send_command_to_motors(&cmd)
                        }
                    }
                };
            }
        }
    }
}

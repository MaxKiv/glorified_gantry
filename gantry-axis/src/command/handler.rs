use gantry_cia402::driver::command::MotorCommand;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::*;

use crate::{axis::AxisMotors, command::GantryCommand, setpoint::translator::SetpointTranslator};

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
        x_translator: Option<SetpointTranslator>,
        y_translator: Option<SetpointTranslator>,
        z_translator: Option<SetpointTranslator>,
    ) -> CommandHandle {
        let (cmd_tx, cmd_rx) = mpsc::channel(10);

        let handle = crate::spawn_logged("CMD", async move {
            CommandHandler::handle_commands(
                cmd_rx,
                x_motors,
                y_motors,
                z_motors,
                x_translator,
                y_translator,
                z_translator,
            )
            .await
        });

        CommandHandle { handle, cmd_tx }
    }

    pub async fn handle_commands(
        mut cmd_rx: mpsc::Receiver<GantryCommand>,
        x_axis: Option<AxisMotors>,
        y_axis: Option<AxisMotors>,
        z_axis: Option<AxisMotors>,
        x_translator: Option<SetpointTranslator>,
        y_translator: Option<SetpointTranslator>,
        z_translator: Option<SetpointTranslator>,
    ) -> anyhow::Result<()> {
        let mut cmd_x: Option<MotorCommand>;
        let mut cmd_y: Option<MotorCommand>;
        let mut cmd_z: Option<MotorCommand>;

        loop {
            if let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    GantryCommand::Setpoint { x, y, z } => {
                        info!("Gantry Setpoint received: {x:?}, {y:?}, {z:?}");

                        // Translate setpoints to motor units
                        cmd_x = if let Some(setpoint) = x
                            && let Some(ref translator) = x_translator
                        {
                            Some(translator.to_motor_cmd(setpoint))
                        } else {
                            None
                        };

                        cmd_y = if let Some(setpoint) = y
                            && let Some(ref translator) = y_translator
                        {
                            Some(translator.to_motor_cmd(setpoint))
                        } else {
                            None
                        };

                        cmd_z = if let Some(setpoint) = z
                            && let Some(ref translator) = z_translator
                        {
                            Some(translator.to_motor_cmd(setpoint))
                        } else {
                            None
                        };

                        info!(
                            "Gantry Setpoint translated: {:?}, {:?}, {:?}",
                            cmd_x, cmd_y, cmd_z
                        );

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
                        info!(
                            "Homing gantry: Sending Enable (cia402 transition to Operation Enabled)"
                        );

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

                        info!("Homing gantry: Sending Home");

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

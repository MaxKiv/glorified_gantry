use std::ops::RangeInclusive;

use eframe::{egui};
use gantry_axis::axis::setpoint::{AxisSetpoint, PositionSetpoint, VelocitySetpoint, TorqueSetpoint};
use gantry_axis::axis::Axis;
use gantry_axis::command::GantryCommand;
use gantry_axis::OperationMode;
use tokio::sync::broadcast;

/// Default velocity for profile position = 1mm/s
const POSITION_DEFAULT_VELOCITY_MS: f64 = 0.001;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create broadcast channel for sending commands
    let (tx, _rx) = broadcast::channel::<GantryCommand>(16);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Gantry Control",
        options,
        Box::new(|_cc| Box::new(GuiApp::new(tx))),
    )?;

    Ok(())
}

struct GuiApp {
    tx: broadcast::Sender<GantryCommand>,
    x: f32,
    y: f32,
    z: f32,
    mode: gantry_axis::OperationMode,
}

impl GuiApp {
    fn new(tx: broadcast::Sender<GantryCommand>) -> Self {
        Self {
            tx,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            mode: OperationMode::ProfileVelocity, // default mode
        }
    }

    fn send_setpoints(&self) {
        let setpoints = [(Axis::X, self.x), (Axis::Y, self.y), (Axis::Z, self.z)];

        

        let cmd = GantryCommand::Setpoint {
            x: Some(AxisSetpoint::),
            y: (),
            z: (),
        };

        for (axis, value) in setpoints {
            let cmd = GantryCommand::Setpoint {
                axis,
                mode: self.mode,
                value: value as f64,
            };
            let _ = self.tx.send(cmd);
        }
    }
}

impl epi::App for GuiApp {
    fn name(&self) -> &str {
        "Gantry Axis GUI"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Gantry Axis Control");

            ui.separator();

            ui.label("Operation Mode:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.mode,
                        OperationMode::ProfilePosition, "Position");
                    ui.selectable_value(&mut self.mode,
                        OperationMode::ProfileVelocity, "Velocity");
                    ui.selectable_value(&mut self.mode,
                        OperationMode::ProfileTorque, "Torque");
                });

            ui.add_space(10.0);

            ui.label("X Axis");
            ui.add(egui::Slider::new(&mut self.x, get_mode_range(self.mode,
                Axis::X)).text("X"));

            ui.label("Y Axis");
            ui.add(egui::Slider::new(&mut self.y, get_mode_range(self.mode,
                Axis::Y)).text("Y"));

            ui.label("Z Axis");
            ui.add(egui::Slider::new(&mut self.z, get_mode_range(self.mode,
                Axis::Z)).text("Z"));

            ui.add_space(10.0);

            if ui.button("Send Setpoints").clicked() {
                self.send_setpoints();
            }
        });
    }

    fn get_mode_setpoint_cmd(&self) -> GantryCommand {
        let cmd = GantryCommand::Setpoint {
            x: Some(get_mode_axis_setpoint(self.mode, self.x)),
            y: (),
            z: (),
        };

    }
}

    fn get_mode_axis_setpoint(mode: OperationMode, val: f64) -> AxisSetpoint {
        match mode {
            OperationMode::ProfilePosition => {AxisSetpoint::AbsolutePosition(
                PositionSetpoint { target: val, velocity: POSITION_DEFAULT_VELOCITY_MS}
            )
            },
            OperationMode::ProfileVelocity => {AxisSetpoint::Velocity(
            VelocitySetpoint { target: val }
            )
            },
            OperationMode::ProfileTorque => {AxisSetpoint::Torque(
            TorqueSetpoint { target: val }
            )
            },
            _ => {AxisSetpoint::Velocity(
            VelocitySetpoint { target: 0.0 }
            )
            },
        }
    }

fn get_mode_range(mode: OperationMode, axis: Axis) -> RangeInclusive<f64>{
    match (mode, axis) {
        (OperationMode::ProfilePosition, Axis::X) => {
            -100.0..=100.0
        }
        (OperationMode::ProfileVelocity, Axis::X) => {
            -100.0..=100.0
        }
        (OperationMode::ProfileTorque, Axis::X) => {
            -100.0..=100.0
        }
        (OperationMode::ProfilePosition, Axis::Y) => {
            -100.0..=100.0
        }
        (OperationMode::ProfileVelocity, Axis::Y) => {
            -100.0..=100.0
        }
        (OperationMode::ProfileTorque, Axis::Y) => {
            -100.0..=100.0
        }
        (OperationMode::ProfilePosition, Axis::Z) => {
            -100.0..=100.0
        }
        (OperationMode::ProfileVelocity, Axis::Z) => {
            -100.0..=100.0
        }
        (OperationMode::ProfileTorque, Axis::Z) => {
            -100.0..=100.0
        }
        _ => ()
    }
}


use std::ops::RangeInclusive;

use eframe::egui;
use gantry_axis::{
    OperationMode,
    axis::{Axis, setpoint::*},
    command::GantryCommand,
};
use tokio::sync::broadcast;
use uom::si::{
    f64::{Length, Torque, Velocity},
    length::millimeter,
    torque::newton_meter,
    velocity::meter_per_second,
};

/// Default velocity for profile position = 1 mm/s
const POSITION_DEFAULT_VELOCITY_MS: f64 = 0.001;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Broadcast channel for sending GantryCommands
    let (tx, _rx) = broadcast::channel::<GantryCommand>(16);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Gantry Control",
        options,
        Box::new(|_cc| Ok(Box::new(GuiApp::new(tx)))),
    )
    .expect("Failed to start eframe window");

    Ok(())
}

struct GuiApp {
    tx: broadcast::Sender<GantryCommand>,
    x: f32,
    y: f32,
    z: f32,
    mode: OperationMode,
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
        let cmd = GantryCommand::Setpoint {
            x: Some(get_mode_axis_setpoint(self.mode, self.x)),
            y: Some(get_mode_axis_setpoint(self.mode, self.y)),
            z: Some(get_mode_axis_setpoint(self.mode, self.z)),
        };

        if let Err(e) = self.tx.send(cmd) {
            eprintln!("Unable to send setpoint: {e}");
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Gantry Axis Control");
            ui.separator();

            ui.label("Operation Mode:");
            egui::ComboBox::from_id_salt("mode_select")
                .selected_text(format!("{:?}", self.mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.mode, OperationMode::ProfilePosition, "Position");
                    ui.selectable_value(&mut self.mode, OperationMode::ProfileVelocity, "Velocity");
                    ui.selectable_value(&mut self.mode, OperationMode::ProfileTorque, "Torque");
                });

            ui.add_space(10.0);

            ui.add(egui::Slider::new(&mut self.x, get_mode_range(self.mode, Axis::X)).text("X"));
            ui.add(egui::Slider::new(&mut self.y, get_mode_range(self.mode, Axis::Y)).text("Y"));
            ui.add(egui::Slider::new(&mut self.z, get_mode_range(self.mode, Axis::Z)).text("Z"));

            ui.add_space(10.0);

            if ui.button("Send Setpoints").clicked() {
                self.send_setpoints();
            }
        });
    }
}

fn get_mode_axis_setpoint(mode: OperationMode, val: f32) -> AxisSetpoint {
    match mode {
        OperationMode::ProfilePosition => AxisSetpoint::AbsolutePosition(PositionSetpoint {
            target: Length::new::<millimeter>(val as f64),
            velocity: Velocity::new::<meter_per_second>(POSITION_DEFAULT_VELOCITY_MS),
        }),
        OperationMode::ProfileVelocity => AxisSetpoint::Velocity(VelocitySetpoint {
            target: Velocity::new::<meter_per_second>(val as f64),
        }),
        OperationMode::ProfileTorque => AxisSetpoint::Torque(TorqueSetpoint {
            target: Torque::new::<newton_meter>(val as f64),
        }),
        _ => AxisSetpoint::Velocity(VelocitySetpoint {
            target: Velocity::new::<meter_per_second>(0.0),
        }),
    }
}

fn get_mode_range(mode: OperationMode, axis: Axis) -> RangeInclusive<f32> {
    match (mode, axis) {
        (OperationMode::ProfilePosition, Axis::X) => -100.0..=100.0,
        (OperationMode::ProfileVelocity, Axis::X) => -100.0..=100.0,
        (OperationMode::ProfileTorque, Axis::X) => -100.0..=100.0,
        (OperationMode::ProfilePosition, Axis::Y) => -100.0..=100.0,
        (OperationMode::ProfileVelocity, Axis::Y) => -100.0..=100.0,
        (OperationMode::ProfileTorque, Axis::Y) => -100.0..=100.0,
        (OperationMode::ProfilePosition, Axis::Z) => -100.0..=100.0,
        (OperationMode::ProfileVelocity, Axis::Z) => -100.0..=100.0,
        (OperationMode::ProfileTorque, Axis::Z) => -100.0..=100.0,
        _ => 0.0..=0.0,
    }
}

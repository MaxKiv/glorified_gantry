use crate::{
    axis::{error::AxisError, scaling::AxisScaling},
    cia402::Cia402Identifier,
    oms::{OperationMode, setpoint::Setpoint},
    rt::{engine::cfg::Axis, motor::Cia402Motor},
};

pub mod error;
pub mod scaling;

pub struct GantryAxis {
    pub axis: Axis,
    pub master: Cia402Motor,
    pub slave: Option<Cia402Motor>,
    pub scaling: AxisScaling,
    pub opmode: OperationMode,
}

impl GantryAxis {
    pub fn new(
        axis: Axis,
        master: Cia402Identifier,
        slave: Option<Cia402Identifier>,
        scaling: AxisScaling,
    ) -> Self {
        // Start master
        let master = Cia402Motor::new(master);
        let slave = slave.map(|s| Cia402Motor::new(s));
        let opmode = OperationMode::default();

        Self {
            axis,
            master,
            slave,
            scaling,
            opmode,
        }
    }

    fn get_axis_motors_mut(&mut self) -> impl Iterator<Item = &mut Cia402Motor> {
        [Some(&mut self.master), self.slave.as_mut()]
            .into_iter()
            .flatten()
    }

    pub fn switch_opmode(&mut self, new_opmode: &OperationMode) -> Result<(), AxisError> {
        for motor in self.get_axis_motors_mut() {
            motor
                .switch_operation_mode(new_opmode)
                .map_err(|_| AxisError::UnableToSwitchOpMode(*new_opmode))?;
        }

        self.opmode = *new_opmode;
        Ok(())
    }

    /// Push a new setpoint to the axis master drive, and slave if any
    pub fn new_axis_setpoint(&mut self, setpoint: Setpoint) {
        if let Some(slave) = self.slave.as_mut() {
            self.master.new_motor_setpoint(setpoint.clone());
            slave.new_motor_setpoint(setpoint);
        } else {
            // Avoid setpoint clone
            self.master.new_motor_setpoint(setpoint);
        }
    }
}

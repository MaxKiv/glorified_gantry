use crate::{
    axis::{error::AxisError, scaling::AxisScaling},
    canopen::{CanOpen, sdo::manager::SdoCommand},
    cia402::Cia402Identifier,
    consts::pdo::NodePdoConfig,
    oms::{OperationMode, setpoint::Setpoint},
    rt::{
        engine::cfg::{Axis, AxisConfig},
        motor::Cia402Motor,
    },
};

pub mod error;
pub mod scaling;

pub struct GantryAxis {
    pub axis: Axis,
    pub master: Cia402Motor,
    pub slave: Option<Cia402Motor>,
    pub scaling: AxisScaling,
    pub opmode: OperationMode,
    pdo_cfg: &'static NodePdoConfig, // All motors in axis have the same const pdo_cfg
    default_parameters: &'static [SdoCommand],
}

impl GantryAxis {
    pub fn new(
        axis: Axis,
        master: Cia402Identifier,
        slave: Option<Cia402Identifier>,
        canopen: CanOpen,
        scaling: AxisScaling,
        pdo_cfg: &'static NodePdoConfig,
        default_parameters: &'static [SdoCommand],
    ) -> Self {
        // Start master
        let master = Cia402Motor::new(master, canopen, pdo_cfg, default_parameters);
        let slave = slave.map(|s| Cia402Motor::new(s, canopen, pdo_cfg, default_parameters));
        let opmode = OperationMode::default();

        Self {
            axis,
            master,
            slave,
            scaling,
            opmode,
            pdo_cfg,
            default_parameters,
        }
    }

    /// Get mutable ref to motors that make up this axis
    /// NOTE: always yields &[master, slave] in order
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
        // TODO: scale setpoint using AxisScaling

        if let Some(slave) = self.slave.as_mut() {
            self.master.new_motor_setpoint(setpoint.clone());
            slave.new_motor_setpoint(setpoint);
        } else {
            // Avoid setpoint clone
            self.master.new_motor_setpoint(setpoint);
        }
    }

    pub fn default_parametrisation(&mut self) -> Result<(), AxisError> {
        for motor in self.get_axis_motors_mut() {
            motor
                .default_parametrisation()
                .map_err(|_| AxisError::UnableToDefaultParametrise(motor.id.clone()));
        }

        Ok(())
    }

    pub fn tick(&mut self) {
        for motor in self.get_axis_motors_mut() {
            motor.tick();
        }
    }
}

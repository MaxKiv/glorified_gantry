use crate::{
    axis::scaling::AxisScaling,
    cia402::Cia402Identifier,
    oms::OperationMode,
    rt::{engine::cfg::Axis, motor::Cia402Motor},
};

pub mod scaling;

pub struct GantryAxis {
    pub axis: Axis,
    pub master: Cia402Motor,
    pub slave: Option<Cia402Motor>,
    pub scaling: AxisScaling,
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

        Self {
            axis,
            master,
            slave,
            scaling,
        }
    }

    fn get_axis_motors_mut(&mut self) -> impl Iterator<Item = &mut Cia402Motor> {
        [Some(&mut self.master), self.slave.as_mut()]
            .into_iter()
            .flatten()
    }

    pub fn switch_opmode(&mut self, new_opmode: OperationMode) -> Result<(), ()> {
        for motor in self.get_axis_motors_mut() {
            motor.switch_opmode(new_opmode)?;
        }

        Ok(())
    }
}

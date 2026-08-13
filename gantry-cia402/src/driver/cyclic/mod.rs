use crate::driver::oms::OperationMode;

#[derive(Debug, Clone)]
pub enum CyclicSynchronousMode {
    Position,
    Velocity,
    Torque,
}

#[derive(Debug)]
pub struct ModeConversionError;

impl TryFrom<OperationMode> for CyclicSynchronousMode {
    type Error = ModeConversionError;

    fn try_from(value: OperationMode) -> Result<Self, ModeConversionError> {
        match value {
            OperationMode::CyclicSynchronousPosition => Ok(CyclicSynchronousMode::Position),
            OperationMode::CyclicSynchronousVelocity => Ok(CyclicSynchronousMode::Velocity),
            OperationMode::CyclicSynchronousTorque => Ok(CyclicSynchronousMode::Torque),
            _ => Err(ModeConversionError),
        }
    }
}

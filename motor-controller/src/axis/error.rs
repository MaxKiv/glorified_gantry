use crate::oms::OperationMode;

#[derive(Debug, thiserror::Error)]
pub enum AxisError {
    #[error("Unable to switch into operation mode: {0:?}")]
    UnableToSwitchOpMode(OperationMode),
}

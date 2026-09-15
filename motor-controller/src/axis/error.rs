use crate::{cia402::Cia402Identifier, oms::OperationMode};

#[derive(Debug, thiserror::Error)]
pub enum AxisError {
    #[error("Unable to switch into operation mode: {0:?}")]
    UnableToSwitchOpMode(OperationMode),
    #[error("Unable to default parametrise motor {0:?}")]
    UnableToDefaultParametrise(Cia402Identifier),
}

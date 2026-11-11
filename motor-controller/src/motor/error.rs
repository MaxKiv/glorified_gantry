use crate::{
    canopen::{
        CanOpenError,
        sdo::{
            SdoError,
            manager::{SdoFailure, SdoManagerError},
        },
    },
    motor::script::{Script, ScriptStep},
};

#[derive(Debug, thiserror::Error)]
pub enum MotorError {
    #[error("Unable to remap PDO: {0:?}")]
    PdoRemapping(CanOpenError),
    #[error("Unable to default parametrise motor: {0:?}")]
    UnableToDefaultParametrise(CanOpenError),
    #[error("Sdo Error in step {0:?} of script {1:?}")]
    ScriptSdoError(ScriptStep, Script, SdoFailure),
    #[error("Timeout in step {0:?} of script {1:?}")]
    ScriptTimeout(ScriptStep, Script),
    #[error("CANOpen error {0:?} in step {1:?} of script {2:?}")]
    ScriptCANOpenError(CanOpenError, ScriptStep, Script),
    #[error("SDO manager error: {:?} in step {1:?} of script {2:?}")]
    SDOManagerError(SdoManagerError, ScriptStep, Script),
    #[error("Generic Logic Error")]
    Logic,
}

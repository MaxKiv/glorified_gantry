use crate::error::DriveError;

pub struct StatusWord {}

/// Abstract CANopen trait providing access CANOpen Objects that CiA402 requires
/// These are objects like the controlword, statusword and target position
/// Typically these are mapped onto a PDO, but they can also be down/uploaded using SDO calls
/// Could be implemented using SDO, PDO, C FFI to CANopenNode, etc.
#[async_trait::async_trait]
pub trait Cia402Transport {
    async fn write_controlword(&self, cw: u16) -> Result<(), DriveError>;
    async fn read_statusword(&self) -> Result<u16, DriveError>;
    async fn write_operation_mode(&self, mode: u8) -> Result<(), DriveError>;
    async fn read_operation_mode_display(&self) -> Result<u8, DriveError>;
}

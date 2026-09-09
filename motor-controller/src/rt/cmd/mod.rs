use crate::{canopen::frame::NodeId, oms::OperationMode, rt::RtSetpoint};

pub mod channel;
pub mod queue;

#[derive(Clone, Copy, Debug)]
pub struct ReconfigurePayload {
    pub motor: NodeId,
    pub operation_mode: OperationMode,
}

#[derive(Clone, Copy, Debug)]
pub enum RtCommand {
    Idle,
    Shutdown,
    Reconfigure(ReconfigurePayload),
    SingleCycle(RtSetpoint),
    Cyclic(RtSetpoint),
}

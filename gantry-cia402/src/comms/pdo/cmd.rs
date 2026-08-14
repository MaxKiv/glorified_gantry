use crate::driver::{
    cyclic::CyclicSynchronousMode,
    oms::{OperationMode, home::HomeFlagsCW, position::PositionFlagsCW, setpoint::Setpoint},
    state::Cia402Flags,
};

#[derive(Debug, Clone)]
pub enum PdoCommand {
    WriteCia402Transition(Cia402Flags),
    UpdateCia402Flags(Cia402Flags),
    WriteSetpoint(Setpoint),
    UpdatePositionSetpointFlags(PositionFlagsCW),
    UpdateHomingSetpointFlags(HomeFlagsCW),
    SwitchToCyclicSynchronousMode(CyclicSynchronousMode),
    ExitCyclicSynchronousMode(OperationMode),
}

use crate::driver::{
    cyclic::CyclicSynchronousMode,
    oms::{home::HomeFlagsCW, position::PositionFlagsCW, setpoint::Setpoint},
    state::Cia402Flags,
};

pub enum PdoCommand {
    WriteCia402Transition(Cia402Flags),
    UpdateCia402Flags(Cia402Flags),
    WriteSetpoint(Setpoint),
    UpdatePositionSetpointFlags(PositionFlagsCW),
    UpdateHomingSetpointFlags(HomeFlagsCW),
    SwitchToCyclicSynchronousMode(CyclicSynchronousMode),
    ExitCyclicSynchronousMode,
}

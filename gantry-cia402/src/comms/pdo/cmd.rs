use crate::driver::{cyclic::CyclicSynchronousMode, oms::setpoint::Setpoint, state::Cia402Flags};

pub enum PdoCommand {
    WriteCia402Transition(Cia402Flags),
    WriteSetpoint(Setpoint),
    SwitchToCyclicSynchronousMode(CyclicSynchronousMode),
    ExitCyclicSynchronousMode,
}

use crate::driver::{oms::setpoint::Setpoint, state::Cia402Flags};

pub enum PdoCommand {
    WriteCia402Transition(Cia402Flags),
    WriteSetpoint(Setpoint),
}

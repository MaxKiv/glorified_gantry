use gantry_cia402::driver::{nmt::NmtState, state::Cia402State};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisState {
    Nmt(NmtState),
    Cia402(Cia402State),
}

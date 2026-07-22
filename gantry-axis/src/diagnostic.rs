use gantry_cia402::driver::receiver::{StatusWord, parse::sdo_response::SdoResponse};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticContent {
    StatusWordUpdate(StatusWord),
    SdoResponse(SdoResponse),
    CommunicationLost,
    FaultCleared,
}

#[derive(Debug, Clone)]
pub enum DiagnosticLevel {
    Ok,
    Warn,
    Error,
    Stale,
}

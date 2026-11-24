#[derive(Debug, Clone, PartialEq)]
pub enum GantryEvent {
    Configuring,
    Operational,
    StateFeedback,
    Error,
}

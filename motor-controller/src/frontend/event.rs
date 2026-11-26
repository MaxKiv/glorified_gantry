use crate::rt::feedback::CycleFeedback;

#[derive(Debug, Clone, PartialEq)]
pub enum GantryEvent {
    Configuring,
    Operational,
    StateFeedback,
    Error,
    Feedback(CycleFeedback),
}

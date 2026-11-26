use crate::axis::feedback::AxisFeedback;

#[derive(Debug, Clone, PartialEq)]
pub struct GantryFeedback {
    pub x: Option<AxisFeedback>,
    pub y: Option<AxisFeedback>,
    pub z: Option<AxisFeedback>,
}

use crate::{
    gantry::{feedback::GantryFeedback, skew::GantrySkew},
    rt::timekeeper::CycleTiming,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CycleFeedback {
    cycle: CycleTiming,
    skew: GantrySkew,
    gantry: GantryFeedback,
}
impl CycleFeedback {
    pub fn new(cycle: CycleTiming, skew: GantrySkew, gantry: GantryFeedback) -> Self {
        Self {
            cycle,
            skew,
            gantry,
        }
    }
}

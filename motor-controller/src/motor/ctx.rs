use std::time::Instant;

use crate::canopen::CanOpen;

pub struct TickCtx<'a> {
    pub now: Instant,
    pub can: &'a mut CanOpen,
}

impl<'a> TickCtx<'a> {
    pub fn new(can: &'a mut CanOpen) -> Self {
        Self {
            now: Instant::now(),
            can,
        }
    }
}

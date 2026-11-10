use crate::motor::script::{Script, ScriptStep};

/// Steps to take for ESTOP
pub const ESTOP_STEPS: [ScriptStep; 1] = [ScriptStep::QuickStop];

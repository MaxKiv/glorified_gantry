#[derive(Debug, Clone, PartialEq, Default)]
pub struct PositionModeFeedback {
    pub target_reached: bool,
    pub limit_exceeded: bool,
    pub setpoint_acknowlegded: bool,
    pub following_error: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct HomingFeedback {
    pub at_home: bool,
    pub homing_completed: bool,
    pub homing_error: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VelocityModeFeedback {
    pub speed_is_zero: bool,
    pub deviation_error: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TorqueModeFeedback {
    pub axis_braked: bool,
    pub setpoint_reached: bool,
    pub limit_exceeded: bool,
}

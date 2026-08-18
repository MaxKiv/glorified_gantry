pub struct RealTimeSetpoint {
    generation: u64,
    controlword: ControlWord,
    target: i32,
    // OR?
    // target: MotorSetpoint(ProfilePosition(10))
}

pub struct RealTimeConfig {
    pdo_mapping: PdoMapping,
}

pub struct RealTimeComms {
    event_rx: RT_API::Receiver<RealTimeFeedback>,
}

struct RealTimeFeedback<const N: usize> {
    cycle: u64,
    motors: [MotorFeedback; N],
    timing: CycleTiming,
    errors: RtErrors,
    skew: Option<f64>,
}

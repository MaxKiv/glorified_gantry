enum CyclePhase {
    WaitingForTpdos,
    Ready,
    TimedOut,
}

pub struct CycleState<const N: usize> {
    cycle: u64,
    phase: CyclePhase,
    received: [bool; N],
}

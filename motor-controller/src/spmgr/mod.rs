use std::sync::atomic::AtomicUsize;

pub struct MotorSetpoint {

}
pub struct GantrySetpoint {

}

pub struct SetpointManager {
    setpoint: [MaybeUninit<GantrySetpoint>; 2]
    active: AtomicUsize,
}

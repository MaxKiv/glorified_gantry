#[derive(Debug, Clone, Copy)]
pub struct MotorLimits {
    pub profile_velocity: u32,      // Counts/s
    pub profile_acceleration: u32,  // Counts/s^2
    pub profile_decceleration: u32, // Counts/s^2
}

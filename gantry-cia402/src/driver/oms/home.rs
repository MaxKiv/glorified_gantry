bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    pub struct HomeFlags: u16 {
        const NEW_SETPOINT       = 1 << 4; // Bit 4: Rising edge triggers start of movement
    }
}

impl Default for HomeFlags {
    fn default() -> Self {
        HomeFlags::NEW_SETPOINT
    }
}

#[derive(Clone, Debug)]
pub struct HomingSetpoint {
    pub flags: HomeFlags,
}

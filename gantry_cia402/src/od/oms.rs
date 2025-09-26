use tokio::sync::mpsc::Sender;

pub const DEFAULT_OPERATIONAL_MODE_SETPOINT: Setpoint =
    Setpoint::ProfilePosition(DEFAULT_POSITION_MODE_SETPOINT);
const DEFAULT_POSITION_MODE_SETPOINT: PositionSetpoint = PositionSetpoint {
    flags: PositionModeFlags::empty(),
    target: 0,
    profile_velocity: 0,
};

bitflags::bitflags! {
#[derive(Clone, Copy, Debug)]
    pub struct PositionModeFlags: u16 {
        const NEW_SETPOINT       = 1 << 4; // Bit 4: Rising edge triggers start of movement
        const CHANGE_IMMEDIATELY = 1 << 5; // Bit 5: Should the motor instantly adapt to the new setpoint, or first reach the previous target?
        const RELATIVE           = 1 << 6; // Bit 6: Interpret this target as a relative position, see 0x60F2
        const HALT               = 1 << 8; // Bit 8: Halt the motor
        const CHANGE_ON_SETPOINT = 1 << 9; // Bit 9: Should the motor have velocity 0 at target position? see page 60
    }
}

#[derive(Clone, Debug)]
pub enum Setpoint {
    ProfilePosition(PositionSetpoint),
    ProfileVelocity(VelocitySetpoint),
    ProfileTorque(TorqueSetpoint),
}

#[derive(Clone, Debug)]
pub struct PositionSetpoint {
    pub flags: PositionModeFlags,
    pub target: u32,
    pub profile_velocity: u32,
}

#[derive(Clone, Debug)]
pub struct VelocitySetpoint {
    // TODO uom?
    target: u32,
}

#[derive(Clone, Debug)]
pub struct TorqueSetpoint {
    // TODO uom?
    target: u32,
}

pub struct OperationModeSpecificHandler {
    new_oms_setpoint_sender: Sender<Setpoint>,
}

impl OperationModeSpecificHandler {
    pub fn new(new_oms_setpoint_sender: Sender<Setpoint>) -> Self {
        Self {
            new_oms_setpoint_sender,
        }
    }

    pub fn set_mode_setpoint(&mut self, new_oms_setpoint: Setpoint) {
        // Validate new setpoint

        // Send it along
        self.new_oms_setpoint_sender.send(new_oms_setpoint.clone());
    }
}

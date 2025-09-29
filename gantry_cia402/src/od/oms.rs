use tokio::sync::mpsc::Sender;

use crate::{driver::update::Update, error::DriveError, od::drive_publisher::Update};

pub const STARTUP_SETPOINT: Setpoint = Setpoint::ProfilePosition(STARTUP_POSITIONMODE_SETPOINT);
const STARTUP_POSITIONMODE_SETPOINT: PositionSetpoint = PositionSetpoint {
    flags: PositionModeFlags::empty(),
    target: 0,
    profile_velocity: 0,
};

const DEFAULT_POSITIONMODE_FLAGS: PositionModeFlags = PositionModeFlags::from_bits_truncate(
    PositionModeFlags::NEW_SETPOINT           // By default start movement when new setpoint is given
        | PositionModeFlags::CHANGE_IMMEDIATELY // By default instantly adopt new setpoint, overriding old
        | !(PositionModeFlags::RELATIVE) // By default interpret target position as absolute position
        | PositionModeFlags::CHANGE_ON_SETPOINT, // By default have zero velocity when reaching setpoint
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    ManufacturerSpecific = -1,
    ProfilePosition = 0,
    Velocity = 1,
    ProfileVelocity = 2,
    ProfileTorque = 3,
    Homing = 6,
    InterpolatedPosition = 7,
    CyclicSynchronousPosition = 8,
    CyclicSynchronousVelocity = 9,
    CyclicSynchronousTorque = 10,
}

impl TryFrom<i8> for OperationMode {
    type Error = ();

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(Self::ManufacturerSpecific),
            0 => Ok(Self::ProfilePosition),
            1 => Ok(Self::Velocity),
            2 => Ok(Self::ProfileVelocity),
            3 => Ok(Self::ProfileTorque),
            6 => Ok(Self::Homing),
            7 => Ok(Self::InterpolatedPosition),
            8 => Ok(Self::CyclicSynchronousPosition),
            9 => Ok(Self::CyclicSynchronousVelocity),
            10 => Ok(Self::CyclicSynchronousTorque),
            _ => Err(()),
        }
    }
}

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
    current_setpoint: Setpoint,
    new_oms_setpoint_sender: Sender<Setpoint>,
}

impl OperationModeSpecificHandler {
    pub fn new(new_oms_setpoint_sender: Sender<Setpoint>) -> Self {
        Self {
            current_setpoint: STARTUP_SETPOINT,
            new_oms_setpoint_sender,
        }
    }

    pub async fn halt_motor(&mut self) -> Result<Update, DriveError> {
        // Keep old setpoint, but enable bit 8 to indicate immediate HALT
        match self.current_setpoint {
            Setpoint::ProfilePosition(mut position_setpoint) => {
                position_setpoint.flags |= PositionModeFlags::HALT;
                self.current_setpoint = Setpoint::ProfilePosition(position_setpoint);
            }
            Setpoint::ProfileVelocity(velocity_setpoint) => todo!(),
            Setpoint::ProfileTorque(torque_setpoint) => todo!(),
        }

        Ok(Update::from_setpoint(self.current_setpoint))
    }

    pub async fn move_absolute(
        &mut self,
        target: i32,
        velocity: i32,
    ) -> Result<Update, DriveError> {
        self.current_setpoint = Setpoint::ProfilePosition(PositionSetpoint {
            flags: DEFAULT_POSITIONMODE_FLAGS.flags &= !PositionModeFlags::RELATIVE,
            target,
            profile_velocity: velocity,
        });

        Ok(Update::from_setpoint(self.current_setpoint))
    }

    pub async fn move_relative(&mut self, delta: i32, velocity: i32) -> Update {
        self.current_setpoint = Setpoint::ProfilePosition(PositionSetpoint {
            flags: DEFAULT_POSITIONMODE_FLAGS.flags &= PositionModeFlags::RELATIVE,
            target,
            profile_velocity: velocity,
        });

        Ok(Update::from_setpoint(self.current_setpoint))
    }

    pub async fn move_velocity(&mut self, target_velocity: i32) -> Update {
        self.current_setpoint = Setpoint::ProfileVelocity(VelocitySetpoint {
            target: target_velocity,
        });

        Ok(Update::from_setpoint(self.current_setpoint))
    }

    pub async fn move_torque(&mut self, target_torque: i32) -> Update {
        self.current_setpoint = Setpoint::ProfileTorque(TorqueSetpoint {
            target: target_torque,
        });

        Ok(Update::from_setpoint(self.current_setpoint))
    }
}

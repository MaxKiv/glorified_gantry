pub mod home;
pub mod position;
pub mod setpoint;
pub mod torque;
pub mod velocity;

use home::*;
use position::*;
use torque::*;
use velocity::*;

use crate::driver::{oms::setpoint::Setpoint, receiver::StatusWord};

pub const STARTUP_SETPOINT: Setpoint = Setpoint::ProfilePosition(STARTUP_POSITIONMODE_SETPOINT);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    AutoSetup = -2,
    ClockDirectionMode = -1,
    NoChange = 0,
    ProfilePosition = 1,
    Velocity = 2,
    ProfileVelocity = 3,
    ProfileTorque = 4,
    Reserved = 5,
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
            -2 => Ok(Self::AutoSetup),
            -1 => Ok(Self::ClockDirectionMode),
            0 => Ok(Self::NoChange),
            1 => Ok(Self::ProfilePosition),
            2 => Ok(Self::Velocity),
            3 => Ok(Self::ProfileVelocity),
            4 => Ok(Self::ProfileTorque),
            6 => Ok(Self::Homing),
            7 => Ok(Self::InterpolatedPosition),
            8 => Ok(Self::CyclicSynchronousPosition),
            9 => Ok(Self::CyclicSynchronousVelocity),
            10 => Ok(Self::CyclicSynchronousTorque),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum OMSFlagsSW {
    Homing(HomeFlagsSW),
    ProfilePosition(PositionFlagsSW),
    ProfileVelocity(VelocityFlagsSW),
    ProfileTorque(TorqueFlagsSW),
    None,
}

impl OMSFlagsSW {
    pub fn from_statusword_and_opmode(statusword: StatusWord, opmode: OperationMode) -> Self {
        // Parse Operation Mode Specific bits of the statusword
        match opmode {
            OperationMode::ProfilePosition => {
                OMSFlagsSW::ProfilePosition(PositionFlagsSW::from_status(statusword))
            }
            OperationMode::ProfileVelocity => {
                OMSFlagsSW::ProfileVelocity(VelocityFlagsSW::from_status(statusword))
            }
            OperationMode::ProfileTorque => {
                OMSFlagsSW::ProfileTorque(TorqueFlagsSW::from_status(statusword))
            }
            OperationMode::Homing => OMSFlagsSW::Homing(HomeFlagsSW::from_status(statusword)),
            _ => {
                tracing::trace!("No specific statusword parsing for current opmode {opmode:?}");
                OMSFlagsSW::None
            }
        }
    }
}

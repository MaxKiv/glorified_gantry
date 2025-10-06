use oze_canopen::interface::CanOpenInterface;
use tokio::task::JoinHandle;
use tokio::{
    sync::mpsc::{self},
    task,
};
use tracing::*;

pub const STARTUP_SETPOINT: Setpoint = Setpoint::ProfilePosition(STARTUP_POSITIONMODE_SETPOINT);
const STARTUP_POSITIONMODE_SETPOINT: PositionSetpoint = PositionSetpoint {
    flags: PositionModeFlags::empty(),
    target: 0,
    profile_velocity: 0,
};

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

impl Default for PositionModeFlags {
    fn default() -> Self {
        PositionModeFlags::NEW_SETPOINT// By default start movement when new setpoint is given
        | PositionModeFlags::CHANGE_IMMEDIATELY  // By default instantly adopt new setpoint, overriding old
        | !(PositionModeFlags::RELATIVE)         // By default interpret target position as absolute position
        | !(PositionModeFlags::HALT)             // By default do not halt
        | PositionModeFlags::CHANGE_ON_SETPOINT // By default have zero velocity when reaching setpoint
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
    pub target: i32,
    pub profile_velocity: u32,
}

#[derive(Clone, Debug)]
pub struct VelocitySetpoint {
    // TODO uom?
    pub target_velocity: i32,
}

#[derive(Clone, Debug)]
pub struct TorqueSetpoint {
    // TODO uom?
    pub target_torque: i32,
}

pub struct OmsHandler {
    node_id: u8,
    canopen: CanOpenInterface,
    setpoint_cmd_rx: mpsc::Receiver<Setpoint>,
    setpoint_update_tx: mpsc::Sender<Setpoint>,
}

impl OmsHandler {
    pub fn init(
        node_id: u8,
        canopen: CanOpenInterface,
        setpoint_cmd_rx: mpsc::Receiver<Setpoint>,
        setpoint_update_tx: mpsc::Sender<Setpoint>,
    ) -> JoinHandle<()> {
        let oms_handler = Self {
            node_id,
            canopen,
            setpoint_cmd_rx,
            setpoint_update_tx,
        };

        let oms_handle = task::spawn(oms_handler.run());

        oms_handle
    }

    pub async fn run(mut self) {
        loop {
            // Process Setpoint updates
            if let Some(new_setpoint) = self.setpoint_cmd_rx.recv().await {
                trace!("OMS handler received new setpoint: {new_setpoint:?}",);

                if let Err(err) = self.setpoint_update_tx.send(new_setpoint).await {
                    error!("Failed to send setpoint update to update task: {err}");
                }
            }
        }
    }
}

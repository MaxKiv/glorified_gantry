use tokio::sync::broadcast;
use tokio::sync::mpsc;

/// Non-RT part
/// Responsible for
///  configuration             
///  external commands        
///  logging/UI/RPC          

pub struct PositionCommand {
    target: Length,
    vel: Speed,
    relative: bool,
}

pub struct VelocityCommand {
    target: Speed,
}

pub struct TorqueCommand {
    target: Torque,
}

pub enum AxisSetpoint {
    Position(PositionCommand),
    Velocity(VelocityCommand),
    Torque(TorqueCommand),
}

pub struct GantrySetpoint {
    x: Option<AxisSetpoint>,
    y: Option<AxisSetpoint>,
    z: Option<AxisSetpoint>,
}

pub enum GantryCommand {
    Setpoint(GantrySetpoint),
    Home,
    Stop,
    Enable,
    Disable,
    ResetFaults,
}

/// https://excalidraw.com/
pub enum GantryEvent {}

// | Thing                      | Owner                                         |
// | -------------------------- | --------------------------------------------- |
// | CAN socket                 | RT                                            |
// | PDO RX/TX                  | RT                                            |
// | SYNC                       | RT                                            |
// | SDO transactions           | non-RT, executed through RT-owned CAN service |   ! <-- I question the need for this, RT does parametrisation?
// | NMT state                  | RT                                            |
// | CiA-402 state              | RT                                            |
// | controlword generation     | RT                                            |
// | statusword interpretation  | RT                                            |
// | Profile Position handshake | RT                                            |
// | actual position/velocity   | RT                                            |
// | trajectory execution       | RT                                            |
// | axis synchronization       | RT                                            |
// | machine safety state       | RT                                            |
// | configuration              | non-RT                                        |
// | external commands          | non-RT                                        |
// | logging/UI/RPC             | non-RT                                        |
/// Non-RT part of [`MotionController`]
pub struct GantryController {
    cmd_rx: mpsc::Receiver<GantryCommand>,
    event_tx: broadcast::Sender<GantryEvent>,
}

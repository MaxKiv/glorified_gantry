use crate::{
    canopen::nmt::NmtState, cia402::Cia402State, cw::ControlWord, motor::error::MotorError,
    oms::OperationMode, sw::StatusWord,
};
use uom::si::{
    f64::{Length, Torque, Velocity},
    length::millimeter,
    torque::newton_meter,
    velocity::millimeter_per_second,
};

/// State of the Cia402Motor Controller
pub enum ControllerState {
    Operating,           // Actively controlling motor
    Configuring,         // Paused Operating to change motor configuration
    Faulted(MotorError), // Something bad
}

#[derive(Debug)]
pub struct MotorState {
    pub pos: Length,               // Current position
    pub vel: Velocity,             // Current velocity
    pub torque: Torque,            // Current torque
    pub cia402_state: Cia402State, // Current CiA402 state
    pub nmt: NmtState,             // Current NMT State
    pub opmode: OperationMode,     // Active operation mode
    pub cw: ControlWord,           // Active controlword
    pub sw: StatusWord,            // Last received statusword
}

impl MotorState {
    pub fn new() -> Self {
        Self {
            nmt: NmtState::Bootup,
            opmode: OperationMode::default(),
            cia402_state: Cia402State::default(),
            cw: ControlWord::default(),
            sw: StatusWord::default(),
            pos: Length::new::<millimeter>(0.0),
            vel: Velocity::new::<millimeter_per_second>(0.0),
            torque: Torque::new::<newton_meter>(0.0),
        }
    }
}

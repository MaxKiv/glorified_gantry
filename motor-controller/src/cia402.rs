#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cia402State {
    #[default]
    NotReadyToSwitchOn,
    SwitchOnDisabled,
    ReadyToSwitchOn,
    SwitchedOn,
    OperationEnabled,
    QuickStopActive,
    FaultReactionActive,
    Fault,
}

pub struct Cia402Manager {
    cia402_state: Cia402State,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Cia402Flags: u16 {
        /// Bit 0: Switch on
        /// Requests transition from "Ready to Switch On" → "Switched On".
        const SWITCH_ON        = 1 << 0;

        /// Bit 1: Enable voltage
        /// Powers the drive (main contactor / power stage).
        const ENABLE_VOLTAGE   = 1 << 1;

        /// Bit 2: Quick stop
        /// 0 = initiate quick stop according to deceleration parameters.
        /// 1 = allow operation.
        const DISABLE_QUICK_STOP       = 1 << 2;

        /// Bit 3: Enable operation
        /// Allows motion commands when set, completing transition into "Operation Enabled".
        const ENABLE_OPERATION = 1 << 3;

        /// Bit 7: Fault reset
        /// Rising edge resets faults and attempts to return to "Switch On Disabled".
        const FAULT_RESET      = 1 << 7;
    }
}

/// Identifies a single Cia402 capable motor drive
/// Cia402 drives contain this in OD 0x1000 & 0x1008
#[derive(Debug, Clone)]
pub struct Cia402Identifier {
    pub node_id: NodeId,                         // CAN node id
    pub device_profile_number: CiaProfileNumber, // Supported cia standard
    pub motor_type: MotorType,                   // Type of the motor
    pub device_name: &'static str,               // Device name given by manufacturer
}

impl std::fmt::Display for Cia402Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{} - node_id: {}", self.device_name, self.node_id)
    }
}

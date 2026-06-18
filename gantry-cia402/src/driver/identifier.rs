use oze_canopen::canopen::NodeId;

/// Describes motor type, 2 = BLDC, 4 = Stepper, 6 = Both
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotorType {
    BLDC = 2,
    Stepper = 4,
    Both = 6,
}

#[derive(Debug)]
pub struct InvalidMotorTypeError(pub u16);

impl std::fmt::Display for InvalidMotorTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid motor type: {}", self.0)
    }
}

impl std::error::Error for InvalidMotorTypeError {}

impl TryFrom<u16> for MotorType {
    type Error = InvalidMotorTypeError;

    fn try_from(val: u16) -> std::result::Result<Self, <Self as TryFrom<u16>>::Error> {
        match val {
            2 => Ok(Self::BLDC),
            4 => Ok(Self::Stepper),
            6 => Ok(Self::Both),
            _ => Err(InvalidMotorTypeError(val)),
        }
    }
}

/// Describes the supported CANopen standard, defaults to 402
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CiaProfileNumber {
    Cia402 = 402,
    Bad = 1,
}

#[derive(Debug)]
pub struct InvalidCiaProfileNumber(pub u16);

impl std::fmt::Display for InvalidCiaProfileNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid profile number: {}", self.0)
    }
}

impl std::error::Error for InvalidCiaProfileNumber {}

impl TryFrom<u16> for CiaProfileNumber {
    type Error = InvalidCiaProfileNumber;

    fn try_from(val: u16) -> std::result::Result<Self, <Self as TryFrom<u16>>::Error> {
        match val {
            402 => Ok(Self::Cia402),
            _ => Err(InvalidCiaProfileNumber(val)),
        }
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

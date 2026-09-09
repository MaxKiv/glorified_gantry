#[derive(Debug)]
pub struct EmergencyMessage {
    pub node_id: NodeId,
    pub error: EMCY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EMCY {
    Undervoltage,
    InterlockError,
    SoftwareReset,
    InternalSoftwareError,
    RatedCurrentNotSet,
    BallastResistorOverload,
    MotorBlocked,
    InternalCorrectionFactorMissing,
    Sensor1Fault,
    Sensor2Fault,
    SensorNFault,
    NonvolatileMemoryFull,
    FieldbusError,
    HeartbeatError,
    SlaveTimeout,
    PdoLengthError,
    PdoLengthExceeded,
    UnexpectedSyncLength,
    SpeedMonitoringError,
    FollowingErrorTooLarge,
    LimitSwitchExceeded,
    NoFurtherPendingErrors,
    Unknown,
}

impl EMCY {
    fn from_error_code(error_code: u16) -> EMCY {
        match error_code {
            0x0 => EMCY::NoFurtherPendingErrors,
            0x3100 => EMCY::Undervoltage,
            0x8210 => EMCY::PdoLengthError,
            0x8220 => EMCY::PdoLengthExceeded,
            0x5440 => EMCY::InterlockError,
            0x6010 => EMCY::SoftwareReset,
            0x6100 => EMCY::InternalSoftwareError,
            0x6320 => EMCY::RatedCurrentNotSet,
            0x7113 => EMCY::BallastResistorOverload,
            0x7121 => EMCY::MotorBlocked,
            0x7200 => EMCY::InternalCorrectionFactorMissing,
            0x7305 => EMCY::Sensor1Fault,
            0x7306 => EMCY::Sensor2Fault,
            0x7307 => EMCY::SensorNFault,
            0x7600 => EMCY::NonvolatileMemoryFull,
            0x8100 => EMCY::FieldbusError,
            0x8130 => EMCY::HeartbeatError,
            0x8200 => EMCY::SlaveTimeout,
            0x8240 => EMCY::UnexpectedSyncLength,
            0x8400 => EMCY::SpeedMonitoringError,
            0x8611 => EMCY::FollowingErrorTooLarge,
            0x8612 => EMCY::LimitSwitchExceeded,
            _ => EMCY::Unknown,
        }
    }
}

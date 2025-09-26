use tracing::*;

use crate::od::{
    drive_publisher::ControlWord,
    err::FaultMessage,
    oms::{PositionModeFlags, PositionSetpoint, Setpoint},
    state::Cia402State,
};

/// Indicates the controlword bits that need to be set and cleared for Cia402State transitions
#[derive(Debug)]
pub struct BitMask {
    pub set: u16,
    pub clear: u16,
}

impl BitMask {
    /// Defines the bits to be set and cleared for each Cia402State
    /// Follows page 46 of PD4C_CANopen_Technical-Manual_V3.3.0
    pub fn get_controlword_mask_for_state(state: &Cia402State) -> Self {
        match state {
            // Unreachable state: Device boots into SwitchOnDisabled
            Cia402State::NotReadyToSwitchOn => BitMask {
                set: 0,
                clear: (1 << 7) | (1 << 3) | (1 << 2) | (1 << 1) | (1 << 0),
            },
            Cia402State::SwitchOnDisabled => BitMask {
                set: 0,
                clear: (1 << 7) | (1 << 1),
            },
            Cia402State::ReadyToSwitchOn => BitMask {
                set: (1 << 2) | (1 << 1),
                clear: (1 << 7) | (1 << 0),
            },
            Cia402State::SwitchedOn => BitMask {
                set: (1 << 2) | (1 << 1) | (1 << 0),
                clear: (1 << 7) | (1 << 3),
            },
            Cia402State::OperationEnabled => BitMask {
                set: (1 << 3) | (1 << 2) | (1 << 1) | (1 << 0),
                clear: (1 << 7),
            },
            Cia402State::QuickStopActive => BitMask {
                set: (1 << 1),
                clear: (1 << 2),
            },
            // Unreachable state: Device switches to this when an Error occurs
            Cia402State::FaultReactionActive => BitMask { set: 0, clear: 0 },
            // Unreachable state: Device switches to this when an Error occurs
            Cia402State::Fault => BitMask { set: 0, clear: 0 },
        }
    }

    pub fn get_controlword_mask_for_oms_setpoint(oms_setpoint: &Setpoint) -> Self {
        match oms_setpoint {
            Setpoint::ProfilePosition(PositionSetpoint { flags, .. }) => {
                trace!("Getting controlword mask for position flags {flags:?}");
                Self::from_position_flags(flags)
            }
            Setpoint::ProfileVelocity(velocity_mode_setpoint) => todo!(),
            Setpoint::ProfileTorque(torque_mode_setpoint) => todo!(),
        }
    }

    const POSITION_MASK: u16 = (PositionModeFlags::NEW_SETPOINT.bits()
        | PositionModeFlags::CHANGE_IMMEDIATELY.bits()
        | PositionModeFlags::RELATIVE.bits()
        | PositionModeFlags::HALT.bits()
        | PositionModeFlags::CHANGE_ON_SETPOINT.bits());

    fn from_position_flags(flags: &PositionModeFlags) -> Self {
        let set = flags.bits();
        let clear = Self::POSITION_MASK & !flags.bits();
        Self { set, clear }
    }

    /// Modify the controlword by setting and clearing the appropriate controlword bits
    /// Follows page 46 of PD4C_CANopen_Technical-Manual_V3.3.0
    pub fn apply_controlword_mask(mask: BitMask, mut controlword: ControlWord) -> ControlWord {
        controlword &= !(mask.clear);
        controlword |= mask.set;
        controlword
    }
}

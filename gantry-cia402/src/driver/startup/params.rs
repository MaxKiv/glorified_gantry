use crate::{comms::sdo::SdoAction, od::*};

pub const PARAMS: &[SdoAction] = &[
    // Always good to upload device type for info
    SdoAction::Upload {
        entry: &DEVICE_TYPE,
    },
    // --- Position-related ---
    // Set target position = 0 (we start from home or zero)
    SdoAction::Download {
        entry: &SET_TARGET_POSITION,
        data: &0i32.to_le_bytes(),
    },
    // Software position limits (disable by using min > max or wide range)
    SdoAction::Download {
        entry: &SOFTWARE_POSITION_RANGE_LIMIT_MIN,
        data: &0i32.to_le_bytes(), // often used as "disable"
    },
    SdoAction::Download {
        entry: &SOFTWARE_POSITION_RANGE_LIMIT_MAX,
        data: &0i32.to_le_bytes(), // often used as "disable"
    },
    SdoAction::Download {
        entry: &HOME_OFFSET,
        data: &0i32.to_le_bytes(),
    },
    SdoAction::Download {
        entry: &POSITION_RANGE_LIMIT_MIN,
        data: &(-36000i32).to_le_bytes(), // 3600 counts = 1 rev
    },
    SdoAction::Download {
        entry: &POSITION_RANGE_LIMIT_MAX,
        data: &(36000i32).to_le_bytes(), // 3600 counts = 1 rev
    },
    // --- Motor behavior / direction ---
    SdoAction::Download {
        entry: &POLARITY,
        data: &0u8.to_le_bytes(), // normal direction
    },
    // --- Profile motion parameters ---
    SdoAction::Download {
        entry: &PROFILE_VELOCITY,
        data: &(30u32).to_le_bytes(), // 30 revs/minute
    },
    SdoAction::Download {
        entry: &END_VELOCITY,
        data: &0u32.to_le_bytes(), // must be 0 for PP mode
    },
    SdoAction::Download {
        entry: &PROFILE_ACCELERATION,
        data: &(20_000u32).to_le_bytes(), // counts/s²
    },
    SdoAction::Download {
        entry: &PROFILE_DECELERATION,
        data: &(20_000u32).to_le_bytes(),
    },
    SdoAction::Download {
        entry: &QUICK_STOP_DECELERATION,
        data: &(30_000u32).to_le_bytes(),
    },
    SdoAction::Download {
        entry: &MOTION_PROFILE_TYPE,
        data: &1i16.to_le_bytes(), // 0 = trapezoidal, 1 = sinusoidal
    },
    SdoAction::Download {
        entry: &MAX_ACCELERATION,
        data: &(30_000u32).to_le_bytes(),
    },
    SdoAction::Download {
        entry: &MAX_DECELERATION,
        data: &(30_000u32).to_le_bytes(),
    },
    SdoAction::Download {
        entry: &POSITIONING_OPTION_CODE,
        data: &0u16.to_le_bytes(), // absolute positioning, immediate start
    },
];

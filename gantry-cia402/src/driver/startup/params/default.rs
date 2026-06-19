use crate::{comms::sdo::SdoAction, driver::startup::home::HomingMethods, od::*};

pub const DEFAULT_PARAMS: &[SdoAction] = &[
    // --- Limit Switches ---
    // NOTE: This causes the drive to error with: Value Out Of Range, wrong datasheet?
    // Forget previous limit switch position
    // SdoAction::Download {
    //     entry: &LIMIT_SWITCH_OPTION_CODE,
    //     data: &(-2i16).to_le_bytes(),
    // },

    // --- Profile Position ---
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
        data: &(0i32).to_le_bytes(), // 3600 counts = 1 rev
    },
    SdoAction::Download {
        entry: &POSITION_RANGE_LIMIT_MAX,
        data: &(0i32).to_le_bytes(), // 3600 counts = 1 rev
    },
    SdoAction::Download {
        entry: &POLARITY,
        data: &0u8.to_le_bytes(), // normal direction
    },
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
    // --- Homing Mode Parameters (CiA 402 § 6.5.1.5) ---
    // 607Ch – Home Offset
    SdoAction::Download {
        entry: &HOME_OFFSET,
        data: &0i32.to_le_bytes(), // controller zero aligns with machine zero
    },
    // 6099h:01h – Speed During Search For Switch
    SdoAction::Download {
        entry: &HOMING_SPEED_SWITCH_SEARCH,
        // data: &0x32u32.to_le_bytes(),
        data: &150u32.to_le_bytes(),
    },
    // 6099h:02h – Speed During Search For Zero
    SdoAction::Download {
        entry: &HOMING_SPEED_ZERO_SEARCH,
        // data: &0x0Au32.to_le_bytes(),
        data: &100u32.to_le_bytes(),
    },
    // 6080h – Max Motor Speed [counts/s]
    SdoAction::Download {
        entry: &MAX_MOTOR_SPEED,
        data: &2000u32.to_le_bytes(),
    },
    // 609Ah – Homing Acceleration
    SdoAction::Download {
        entry: &HOMING_ACCELERATION,
        data: &0x1F4u32.to_le_bytes(),
    },
    // 203Ah:01h – Minimum Current For Block Detection
    SdoAction::Download {
        entry: &BLOCK_DETECTION_MIN_CURRENT,
        data: &0x41Ai32.to_le_bytes(),
    },
    // 203Ah:02h – Period Of Blocking
    SdoAction::Download {
        entry: &BLOCK_DETECTION_PERIOD,
        data: &0xC6i32.to_le_bytes(),
    },
    // --- Profile Torque Mode Parameters (page 70) ---
    // 6072h:00h – Max Torque [‰ of rated torque]
    SdoAction::Download {
        entry: &MAX_TORQUE,
        data: &200u16.to_le_bytes(),
    },
    // 6073h:00h – Max Current [‰ of rated current]
    SdoAction::Download {
        entry: &MAX_CURRENT,
        data: &1000u16.to_le_bytes(),
    },
    // 6087h:00h – Torque Slope [‰/s]
    SdoAction::Download {
        entry: &TORQUE_SLOPE,
        data: &100u32.to_le_bytes(),
    },
    // --- Homing + DI + Limit Switch Behavior ---
    // All of these are required for succesful homing
    // 6098h – Homing Method
    SdoAction::Download {
        entry: &HOMING_METHOD,
        data: &HomingMethods::NegLimitOnly.as_i8().to_le_bytes(),
    },
    // Enable Default Limit Switch Operation: Only note limit switch position
    // Required when homing the motor using limit switches
    SdoAction::Download {
        entry: &LIMIT_SWITCH_OPTION_CODE,
        data: &(-1i16).to_le_bytes(),
    },
    SdoAction::Download {
        entry: &DIGITAL_INPUTS_CONTROL_SPECIAL_FUNCTION,
        data: &(0b01u32).to_le_bytes(), // Enable positive limit switch, Disable Neg limit + homing + interlock -> Datasheet Page 87
    },
    SdoAction::Download {
        entry: &DIGITAL_INPUTS_CONTROL_INVERTED,
        data: &(1u32).to_le_bytes(), // Invert Physical input 1, Required for the limit switch to play nicely during homing
    },
    SdoAction::Download {
        entry: &DIGITAL_INPUTS_ROUTING_ENABLE,
        data: &(1u32).to_le_bytes(), // Enable alternative routing for Digital Inputs
    },
    SdoAction::Download {
        entry: &DIGITAL_INPUTS_ROUTING_2,
        data: &(1u8).to_le_bytes(), // Route physical source 1 -> DI 2 (which is the pos limit switch)
    },
];

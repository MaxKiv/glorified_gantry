use crate::od::ObjectDictionary;

#[derive(Debug)]
pub enum PdoType {
    RPDO,
    TPDO,
}

#[derive(Debug)]
pub struct PdoMapping {
    pub kind: PdoType,
    pub mappings: &'static [PdoMappingSource],
}

#[derive(Debug)]
pub struct PdoMappingSource {
    // OD index to use as source
    pub index: u16,
    // OD subindex to use as source
    pub sub_index: u8,
    // Number of bits of the source to include in the pdo, commonly 16 or 32
    pub number_of_bits: u8,
}

impl PdoMapping {
    pub const DEFAULT_RPDOS: &'static [PdoMapping] = &[Self::RPDO_DEFAULT_1, Self::RPDO_DEFAULT_2];
    pub const DEFAULT_TPDOS: &'static [PdoMapping] = &[Self::TPDO_DEFAULT_1, Self::TPDO_DEFAULT_2];

    pub const CUSTOM_RPDOS: &'static [PdoMapping] = &[
        &Self::RPDO_CONTROL_OPMODE,
        &Self::RPDO_TARGET_POS,
        &Self::RPDO_TARGET_VEL,
        &Self::RPDO_TARGET_TORQUE,
    ];
    // I didn't know of a better const method to do this, seems rust const fn are lacking compared
    // to c++ templates, this never changes anyway
    pub const RPDO_NUM_CONTROL_WORD: usize = 1;
    pub const RPDO_NUM_OPMODE: usize = 1;
    pub const RPDO_NUM_TARGET_POS: usize = 2;
    pub const RPDO_NUM_TARGET_VEL: usize = 3;
    pub const RPDO_NUM_TARGET_TORQUE: usize = 4;

    pub const CUSTOM_TPDOS: &'static [PdoMapping] = &[
        &Self::TPDO_STATUS_OPMODE,
        &Self::TPDO_POS_VEL_ACTUAL,
        &Self::TPDO_TORQUE_ACTUAL,
    ];

    pub const RPDO_DEFAULT_1: PdoMapping = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::CONTROL_WORD.index,
                sub_index: 0x0,
                number_of_bits: 16,
            },
            PdoMappingSource {
                index: ObjectDictionary::SET_OPERATION_MODE.index,
                sub_index: 0x0,
                number_of_bits: 8,
            },
        ],
    };

    pub const RPDO_DEFAULT_2: PdoMapping = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::SET_TARGET_POSITION.index,
                sub_index: 0x0,
                number_of_bits: 32,
            },
            PdoMappingSource {
                index: ObjectDictionary::PROFILE_VELOCITY.index,
                sub_index: 0x0,
                number_of_bits: 32,
            },
        ],
    };

    pub const TPDO_DEFAULT_1: PdoMapping = PdoMapping {
        kind: PdoType::TPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::STATUS_WORD.index,
                sub_index: 0x0,
                number_of_bits: 16,
            },
            PdoMappingSource {
                index: ObjectDictionary::GET_OPERATION_MODE.index,
                sub_index: 0x0,
                number_of_bits: 8,
            },
        ],
    };

    pub const TPDO_DEFAULT_2: PdoMapping = PdoMapping {
        kind: PdoType::TPDO,
        mappings: &[PdoMappingSource {
            index: ObjectDictionary::POSITION_ACTUAL_VALUE.index,
            sub_index: 0x0,
            number_of_bits: 32,
        }],
    };

    pub const RPDO_CONTROL_OPMODE: PdoMapping = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::CONTROL_WORD.index,
                sub_index: ObjectDictionary::CONTROL_WORD.sub_index,
                number_of_bits: 16,
            },
            PdoMappingSource {
                index: ObjectDictionary::SET_OPERATION_MODE.index,
                sub_index: ObjectDictionary::SET_OPERATION_MODE.sub_index,
                number_of_bits: 8,
            },
        ],
    };

    pub const RPDO_TARGET_POS: PdoMapping = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::SET_TARGET_POSITION.index,
                sub_index: ObjectDictionary::SET_TARGET_POSITION.sub_index,
                number_of_bits: 32,
            },
            PdoMappingSource {
                index: ObjectDictionary::PROFILE_VELOCITY.index,
                sub_index: ObjectDictionary::PROFILE_VELOCITY.sub_index,
                number_of_bits: 32,
            },
        ],
    };

    pub const RPDO_TARGET_VEL: PdoMapping = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[PdoMappingSource {
            index: ObjectDictionary::SET_TARGET_VELOCITY.index,
            sub_index: ObjectDictionary::SET_TARGET_VELOCITY.sub_index,
            number_of_bits: 32,
        }],
    };

    pub const RPDO_TARGET_TORQUE: PdoMapping = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[PdoMappingSource {
            index: ObjectDictionary::SET_TARGET_TORQUE.index,
            sub_index: ObjectDictionary::SET_TARGET_TORQUE.sub_index,
            number_of_bits: 32,
        }],
    };

    pub const TPDO_STATUS_OPMODE: PdoMapping = PdoMapping {
        kind: PdoType::TPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::STATUS_WORD.index,
                sub_index: ObjectDictionary::STATUS_WORD.sub_index,
                number_of_bits: 16,
            },
            PdoMappingSource {
                index: ObjectDictionary::GET_OPERATION_MODE.index,
                sub_index: ObjectDictionary::GET_OPERATION_MODE.sub_index,
                number_of_bits: 8,
            },
        ],
    };

    pub const TPDO_POS_VEL_ACTUAL: PdoMapping = PdoMapping {
        kind: PdoType::TPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::POSITION_ACTUAL_VALUE.index,
                sub_index: ObjectDictionary::POSITION_ACTUAL_VALUE.sub_index,
                number_of_bits: 32,
            },
            PdoMappingSource {
                index: ObjectDictionary::VELOCITY_ACTUAL_VALUE.index,
                sub_index: ObjectDictionary::VELOCITY_ACTUAL_VALUE.sub_index,
                number_of_bits: 32,
            },
        ],
    };

    pub const TPDO_TORQUE_ACTUAL: PdoMapping = PdoMapping {
        kind: PdoType::TPDO,
        mappings: &[PdoMappingSource {
            index: ObjectDictionary::TORQUE_ACTUAL_VALUE.index,
            sub_index: ObjectDictionary::TORQUE_ACTUAL_VALUE.sub_index,
            number_of_bits: 32,
        }],
    };
}

/// Returns the COB Id for the given pdo num and type
/// See https://en.wikipedia.org/wiki/CANopen#Process_Data_Object_(PDO)_protocol
pub fn get_pdo_cob_id(pdo_num: usize, kind: PdoType) -> Option<u16> {
    let cob_id = match pdo_num {
        1 => 0x180,
        2 => 0x280,
        3 => 0x380,
        4 => 0x480,
        _ => return None,
    } + pdo_num;

    Some(match kind {
        PdoType::RPDO => cob_id,
        PdoType::TPDO => cob_id + 0x20,
    })
}

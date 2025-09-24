use crate::od::ObjectDictionary;

#[derive(Debug)]
pub enum PdoType {
    RPDO,
    TPDO,
}

#[derive(Debug)]
pub struct PdoMapping<'a> {
    pub kind: PdoType,
    pub mappings: &'a [PdoMappingSource],
}

#[derive(Debug)]
pub struct PdoMappingSource {
    // OD index to use as source
    pub index: u16,
    // OD subindex to use as source
    pub subindex: u8,
    // Number of bits of the source to include in the pdo, commonly 16 or 32
    pub number_of_bits: u8,
}

impl<'a> PdoMapping<'a> {
    pub const DEFAULT_RPDOS: &'a [PdoMapping<'a>] = &[Self::RPDO_DEFAULT_1, Self::RPDO_DEFAULT_2];
    pub const DEFAULT_TPDOS: &'a [PdoMapping<'a>] = &[Self::TPDO_DEFAULT_1, Self::TPDO_DEFAULT_2];

    pub const CUSTOM_RPDOS: &'a [&'a PdoMapping<'a>] = &[
        &Self::RPDO_CUSTOM_1,
        &Self::RPDO_CUSTOM_2,
        &Self::RPDO_CUSTOM_3,
        &Self::RPDO_CUSTOM_4,
    ];
    pub const CUSTOM_TPDOS: &'a [&'a PdoMapping<'a>] = &[
        &Self::TPDO_CUSTOM_1,
        &Self::TPDO_CUSTOM_2,
        &Self::TPDO_CUSTOM_3,
    ];

    pub const RPDO_DEFAULT_1: PdoMapping<'a> = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::CONTROL_WORD.index,
                subindex: 0x0,
                number_of_bits: 16,
            },
            PdoMappingSource {
                index: ObjectDictionary::SET_OPERATION_MODE.index,
                subindex: 0x0,
                number_of_bits: 8,
            },
        ],
    };

    pub const RPDO_DEFAULT_2: PdoMapping<'a> = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::SET_TARGET_POSITION.index,
                subindex: 0x0,
                number_of_bits: 32,
            },
            PdoMappingSource {
                index: ObjectDictionary::PROFILE_VELOCITY.index,
                subindex: 0x0,
                number_of_bits: 32,
            },
        ],
    };

    pub const TPDO_DEFAULT_1: PdoMapping<'a> = PdoMapping {
        kind: PdoType::TPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::STATUS_WORD.index,
                subindex: 0x0,
                number_of_bits: 16,
            },
            PdoMappingSource {
                index: ObjectDictionary::GET_OPERATION_MODE.index,
                subindex: 0x0,
                number_of_bits: 8,
            },
        ],
    };

    pub const TPDO_DEFAULT_2: PdoMapping<'a> = PdoMapping {
        kind: PdoType::TPDO,
        mappings: &[PdoMappingSource {
            index: ObjectDictionary::POSITION_ACTUAL_VALUE.index,
            subindex: 0x0,
            number_of_bits: 32,
        }],
    };

    pub const RPDO_CUSTOM_1: PdoMapping<'a> = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::CONTROL_WORD.index,
                subindex: 0x0,
                number_of_bits: 16,
            },
            PdoMappingSource {
                index: ObjectDictionary::SET_OPERATION_MODE.index,
                subindex: 0x0,
                number_of_bits: 8,
            },
        ],
    };

    pub const RPDO_CUSTOM_2: PdoMapping<'a> = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::CONTROL_WORD.index,
                subindex: 0x0,
                number_of_bits: 16,
            },
            PdoMappingSource {
                index: ObjectDictionary::SET_OPERATION_MODE.index,
                subindex: 0x0,
                number_of_bits: 8,
            },
        ],
    };

    pub const RPDO_CUSTOM_3: PdoMapping<'a> = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[PdoMappingSource {
            index: ObjectDictionary::SET_TARGET_VELOCITY.index,
            subindex: 0x0,
            number_of_bits: 32,
        }],
    };

    pub const RPDO_CUSTOM_4: PdoMapping<'a> = PdoMapping {
        kind: PdoType::RPDO,
        mappings: &[PdoMappingSource {
            index: ObjectDictionary::SET_TARGET_TORQUE.index,
            subindex: 0x0,
            number_of_bits: 32,
        }],
    };

    pub const TPDO_CUSTOM_1: PdoMapping<'a> = PdoMapping {
        kind: PdoType::TPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::STATUS_WORD.index,
                subindex: 0x0,
                number_of_bits: 32,
            },
            PdoMappingSource {
                index: ObjectDictionary::GET_OPERATION_MODE.index,
                subindex: 0x0,
                number_of_bits: 32,
            },
        ],
    };

    pub const TPDO_CUSTOM_2: PdoMapping<'a> = PdoMapping {
        kind: PdoType::TPDO,
        mappings: &[
            PdoMappingSource {
                index: ObjectDictionary::POSITION_ACTUAL_VALUE.index,
                subindex: 0x0,
                number_of_bits: 32,
            },
            PdoMappingSource {
                index: ObjectDictionary::VELOCITY_ACTUAL_VALUE.index,
                subindex: 0x0,
                number_of_bits: 32,
            },
        ],
    };

    pub const TPDO_CUSTOM_3: PdoMapping<'a> = PdoMapping {
        kind: PdoType::TPDO,
        mappings: &[PdoMappingSource {
            index: ObjectDictionary::TORQUE_ACTUAL_VALUE.index,
            subindex: 0x0,
            number_of_bits: 32,
        }],
    };
}

/// Calculates pdo index offset from given base and pdo mapping number
/// For example SDO for Node Id 3 = 0x500 + 3 = 0x503
pub fn calculate_pdo_index_offset(base: u16, pdo_mapping_number: u8) -> u16 {
    base.checked_add(pdo_mapping_number.into())
        .expect("Overflow in RPDO mapping parameter index calculation")
}

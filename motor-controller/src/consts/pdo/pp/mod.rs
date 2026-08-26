use crate::canopen::{
    od,
    pdo::{
        OnSyncN, PdoType, TransmissionType,
        mapping::{OMSNodePdoConfig, PdoMapping, PdoMappingSource},
    },
};

pub const DEFAULT_PP_PDOCFG: OMSNodePdoConfig = OMSNodePdoConfig {
    rpdo1: Some(PdoMapping {
        pdo: PdoType::RPDO,
        sources: &[
            PdoMappingSource::from_od_entry(&od::CONTROL_WORD, 0),
            PdoMappingSource::from_od_entry(&od::SET_OPERATION_MODE, 16),
        ],
        transmission_type: TransmissionType::OnChange,
    }),
    rpdo2: Some(PdoMapping {
        pdo: PdoType::RPDO,
        sources: &[
            PdoMappingSource::from_od_entry(&od::SET_TARGET_POSITION, 0),
            PdoMappingSource::from_od_entry(&od::PROFILE_VELOCITY, 32),
        ],
        transmission_type: TransmissionType::OnChange,
    }),
    tpdo1: Some(PdoMapping {
        pdo: PdoType::TPDO,
        sources: &[
            PdoMappingSource::from_od_entry(&od::STATUS_WORD, 0),
            PdoMappingSource::from_od_entry(&od::GET_OPERATION_MODE, 16),
            PdoMappingSource::from_od_entry(&od::TORQUE_ACTUAL_VALUE, 32),
        ],
        transmission_type: TransmissionType::OnSyncTPDO(OnSyncN::from(1).unwrap()),
    }),
    tpdo2: Some(PdoMapping {
        pdo: PdoType::TPDO,
        sources: &[
            PdoMappingSource::from_od_entry(&od::POSITION_ACTUAL_VALUE, 0),
            PdoMappingSource::from_od_entry(&od::VELOCITY_ACTUAL_VALUE, 32),
        ],
        transmission_type: TransmissionType::OnSyncTPDO(OnSyncN::from(1).unwrap()),
    }),
    ..OMSNodePdoConfig::empty()
};

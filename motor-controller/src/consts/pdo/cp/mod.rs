use crate::canopen::{
    od,
    pdo::{
        OnSyncN, PdoType, TransmissionType,
        mapping::{OMSNodePdoConfig, PdoMapping, PdoMappingSource},
    },
};

pub const DEFAULT_CP_PDOCFG: OMSNodePdoConfig = OMSNodePdoConfig {
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
        sources: &[PdoMappingSource::from_od_entry(&od::SET_TARGET_POSITION, 0)],
        transmission_type: TransmissionType::OnSyncRPDO, // NOTE: this is sent cyclically
    }),
    tpdo1: Some(PdoMapping {
        pdo: PdoType::TPDO,
        sources: &[
            PdoMappingSource::from_od_entry(&od::STATUS_WORD, 0),
            PdoMappingSource::from_od_entry(&od::POSITION_ACTUAL_VALUE, 16),
        ],
        transmission_type: TransmissionType::OnSyncTPDO(OnSyncN::from(1).unwrap()),
    }),
    // TODO:5 determine if below is required
    // tpdo2: Some(PdoMapping {
    //     pdo: PdoType::TPDO,
    //     sources: &[
    //         PdoMappingSource::from_od_entry(&od::VELOCITY_ACTUAL_VALUE, 0),
    //         PdoMappingSource::from_od_entry(&od::FOLLOWING_ERROR_ACTUAL_VALUE, 32),
    //     ],
    //     transmission_type: TransmissionType::OnSyncTPDO(OnSyncN::from(1).unwrap()),
    // }),
    ..OMSNodePdoConfig::empty()
};

use crate::canopen::{
    od,
    pdo::{
        OnSyncN, PdoType, TransmissionType,
        mapping::{OMSNodePdoConfig, PdoMapping, PdoMappingSource},
    },
};

pub const DEFAULT_PT_PDOCFG: OMSNodePdoConfig = OMSNodePdoConfig {
    rpdo: [
        Some(PdoMapping {
            pdo: PdoType::RPDO,
            sources: &[
                PdoMappingSource::from_od_entry(&od::CONTROL_WORD, 0),
                PdoMappingSource::from_od_entry(&od::SET_OPERATION_MODE, 16),
            ],
            transmission_type: TransmissionType::OnChange,
        }),
        Some(PdoMapping {
            pdo: PdoType::RPDO,
            sources: &[PdoMappingSource::from_od_entry(&od::SET_TARGET_TORQUE, 0)],
            transmission_type: TransmissionType::OnSyncRPDO, // NOTE: this is sent cyclically
        }),
        None,
        None,
    ],
    tpdo: [
        Some(PdoMapping {
            pdo: PdoType::TPDO,
            sources: &[
                PdoMappingSource::from_od_entry(&od::STATUS_WORD, 0),
                PdoMappingSource::from_od_entry(&od::GET_OPERATION_MODE, 16),
                PdoMappingSource::from_od_entry(&od::TORQUE_ACTUAL_VALUE, 32),
            ],
            transmission_type: TransmissionType::OnSyncTPDO(OnSyncN::from(1).unwrap()),
        }),
        Some(PdoMapping {
            pdo: PdoType::TPDO,
            sources: &[
                PdoMappingSource::from_od_entry(&od::POSITION_ACTUAL_VALUE, 0),
                PdoMappingSource::from_od_entry(&od::VELOCITY_ACTUAL_VALUE, 32),
            ],
            transmission_type: TransmissionType::OnSyncTPDO(OnSyncN::from(1).unwrap()),
        }),
        None,
        None,
    ],
};

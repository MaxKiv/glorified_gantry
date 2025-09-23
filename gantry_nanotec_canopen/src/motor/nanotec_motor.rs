use std::sync::Arc;

use crate::{
    error::MotorError, motor::manager::MotorManager, od::ObjectDictionary, pdo::PdoMapping,
};
use anyhow::Result;
use oze_canopen::sdo_client::SdoClient;
use tokio::sync::Mutex;
use tracing::*;

/// One CANopen SDO parameter write (or read).
#[derive(Debug)]
pub enum SdoAction<'a> {
    /// Send data to device
    Download {
        index: u16,
        subindex: u8,
        data: &'a [u8],
    },
    /// Fetch data from device
    Upload { index: u16, subindex: u8 },
}

/// Startup parameterization for the Nanotec PD4-C
pub const NANOTEC_PARAMETERS: &[SdoAction] = &[
    // Example: configure receive PDO 1 mapping
    SdoAction::Download {
        index: 0x1801,
        subindex: 2,
        data: &[1u8],
    },
    // Example: request upload for diagnostic
    SdoAction::Upload {
        index: 0x1800,
        subindex: 0,
    },
];

/// Handle to a single Nanotec PD4-C
pub struct NanotecMotor<'a> {
    node_id: u8,
    sdo: Arc<Mutex<SdoClient>>,
    motor_manager: &'a MotorManager,
}

impl<'a> NanotecMotor<'a> {
    pub fn with_node_id(
        node_id: u8,
        sdo: Arc<Mutex<SdoClient>>,
        motor_manager: &'a MotorManager,
    ) -> Self {
        Self {
            node_id,
            sdo,
            motor_manager,
        }
    }

    // TODO: uom distance inputs?
    fn set_position() -> Result<()> {
        todo!();
    }

    /// Configure given PDO mapping
    /// This follows steps listed at page 118 of PD4C_CANopen_Technical_Manual_v3.3
    pub async fn set_pdo_mapping(&self, pdo_mapping: &PdoMapping) -> Result<()> {
        trace!(
            "set_pdo_mapping for nodeId {} to {:?}",
            self.node_id, pdo_mapping
        );

        // 1. Deactivate the PDO by setting the Valid Bit (bit 31) of subindex 01h of the corresponding communication parameter (e.g., 1400h:01h) to "1".
        trace!(
            "1. Deactivate the PDO by setting the Valid Bit (bit 31) of subindex 01h of the corresponding communication parameter (e.g., 1400h:01h) to \"1\".",
        );
        let communication_param = match pdo_mapping.kind {
            crate::pdo::PdoType::RPDO => {
                ObjectDictionary::RPDO_COMMUNICATION_PARAMETER_BASE_INDEX + (pdo_mapping.number - 1)
            }
            crate::pdo::PdoType::TPDO => {
                ObjectDictionary::TPDO_COMMUNICATION_PARAMETER_BASE_INDEX + (pdo_mapping.number - 1)
            }
        };
        let data = [0xFF, 0xFF, 0xFF, 0xFF];
        self.sdo
            .lock()
            .await
            .download(communication_param, 0x1, &data)
            .await
            .map_err(MotorError::CanOpen)?;

        // 2. Deactivate the mapping by setting subindex 00h of the corresponding mapping parameter (e.g., 1600h:00h) to "0".
        let mapping_param = match pdo_mapping.kind {
            crate::pdo::PdoType::RPDO => {
                ObjectDictionary::RPDO_MAPPING_PARAMETER_BASE_INDEX + (pdo_mapping.number - 1)
            }
            crate::pdo::PdoType::TPDO => {
                ObjectDictionary::TPDO_MAPPING_PARAMETER_BASE_INDEX + (pdo_mapping.number - 1)
            }
        };
        let data = [0];
        self.sdo
            .lock()
            .await
            .download(mapping_param, 0x0, &data)
            .await
            .map_err(MotorError::CanOpen)?;

        // 3. Change the mapping in the desired subindices (e.g., 1600h:01h).
        for (number, mapping) in pdo_mapping.mappings.iter().enumerate() {
            let index_be: [u8; 2] = mapping.index.to_be_bytes();
            let data: [u8; 4] = [
                index_be[0],
                index_be[1],
                mapping.subindex,
                mapping.number_of_bits,
            ];
            self.sdo
                .lock()
                .await
                .download(mapping_param, number, &index_be)
                .await
                .map_err(MotorError::CanOpen)?;
        }

        // 4. Activate the mapping by writing the number of objects that are to be mapped in subindex 00h of the corresponding mapping parameter (e.g., 1600h:00h).
        let data = [pdo_mapping.number];
        self.sdo
            .lock()
            .await
            .download(mapping_param, 0x0, &data)
            .await
            .map_err(MotorError::CanOpen)?;

        Ok(())
    }

    pub async fn parametrize(&self) -> Result<()> {
        for action in NANOTEC_PARAMETERS {
            trace!("parametrizing nodeId {}: {action:?}", self.node_id);
            match action {
                SdoAction::Download {
                    index,
                    subindex,
                    data,
                } => {
                    self.sdo
                        .lock()
                        .await
                        .download(*index, *subindex, data)
                        .await
                        .map_err(MotorError::CanOpen)?;
                }
                SdoAction::Upload { index, subindex } => {
                    self.sdo
                        .lock()
                        .await
                        .upload(*index, *subindex)
                        .await
                        .map_err(MotorError::CanOpen)?;
                }
            }
        }

        Ok(())
    }
}

use std::sync::Arc;

use crate::{
    comms::{
        pdo::mapping::{PdoMapping, PdoType, calculate_pdo_index_offset},
        sdo::SdoAction,
        transport::Cia402Transport,
    },
    error::DriveError,
    od::ObjectDictionary,
    state::{Cia402State, Cia402StateMachine},
};

use anyhow::Result;
use oze_canopen::sdo_client::SdoClient;
use tokio::sync::{Mutex, mpsc};
use tracing::*;

/// CiA-402 driver built on top of a CANopen protocol manager
pub struct Drive<T: Cia402Transport> {
    pub node_id: u8,
    state_machine: Cia402StateMachine,
    transport: T,               // SDO or PDO based CANopen control/statusword manager
    sdo: Arc<Mutex<SdoClient>>, // Used for parametrisation
}

impl<T: Cia402Transport> Drive<T> {
    pub fn new(node_id: u8, transport: T, sdo: Arc<Mutex<SdoClient>>) -> Self {
        // TODO: move this up the stack
        let (state_sender, state_receiver): (
            mpsc::Sender<Cia402State>,
            mpsc::Receiver<Cia402State>,
        ) = tokio::sync::mpsc::channel(10);

        Drive {
            node_id,
            state_machine: Cia402StateMachine::new(state_sender),
            transport,
            sdo,
        }
    }

    /// Read the Statusword (0x6041) and update internal state
    pub fn poll_state(&mut self) -> Result<Cia402State, DriveError> {
        let data = self.canopen.sdo_upload(0x6041, 0x00)?;
        let sw = u16::from_le_bytes([data[0], data[1]]);
        self.state = decode_statusword(sw);
        Ok(self.state)
    }

    /// Example: switch on + enable operation
    pub fn enable_operation(&mut self) -> Result<(), DriveError> {
        // Controlword sequence (0x6040) according to CiA-402
        self.canopen
            .sdo_download(0x6040, 0x00, &0x0006u16.to_le_bytes())?; // shutdown
        self.canopen
            .sdo_download(0x6040, 0x00, &0x0007u16.to_le_bytes())?; // switch on
        self.canopen
            .sdo_download(0x6040, 0x00, &0x000Fu16.to_le_bytes())?; // enable op
        self.poll_state()?;
        if self.state != Cia402State::OperationEnabled {
            return Err(DriveError::InvalidTransition(
                self.state,
                Cia402State::OperationEnabled,
            ));
        }
        Ok(())
    }

    /// Configure the devicde with the given set of PDO mappings
    pub async fn configure_pdo_mappings(&self, pdo_mappings: &[&PdoMapping<'_>]) -> Result<()> {
        trace!("configure_pdo_mappings for nodeId {}", self.node_id);
        for (mapping_number, pdo_mapping) in pdo_mappings.iter().enumerate() {
            self.set_pdo_mapping(pdo_mapping, mapping_number as u8)
                .await?;
        }

        Ok(())
    }

    /// Apply the given PDO mapping to the device
    /// This follows steps listed at page 118 of PD4C_CANopen_Technical_Manual_v3.3
    async fn set_pdo_mapping(
        &self,
        pdo_mapping: &PdoMapping<'_>,
        mapping_number: u8,
    ) -> Result<()> {
        trace!(
            "set_pdo_mapping for nodeId {} to {:?}",
            self.node_id, pdo_mapping
        );

        // 1. Deactivate the PDO by setting the Valid Bit (bit 31) of subindex 01h of the corresponding communication parameter (e.g., 1400h:01h) to "1".
        let communication_index = match pdo_mapping.kind {
            PdoType::RPDO => calculate_pdo_index_offset(
                ObjectDictionary::RPDO_COMMUNICATION_PARAMETER_BASE_INDEX,
                mapping_number,
            ),
            PdoType::TPDO => calculate_pdo_index_offset(
                ObjectDictionary::TPDO_COMMUNICATION_PARAMETER_BASE_INDEX,
                mapping_number,
            ),
        };
        trace!(
            "1. Deactivate the PDO by setting the Valid Bit (bit 31) of subindex 01h of the
                corresponding communication parameter ({}) to \"1\".",
            communication_index
        );
        let data = [0xFF, 0xFF, 0xFF, 0xFF];
        self.sdo
            .lock()
            .await
            .download(communication_index, 0x1, &data)
            .await
            .map_err(DriveError::CanOpen)?;

        // 2. Deactivate the mapping by setting subindex 00h of the corresponding mapping parameter to \"0\".,
        let mapping_index = match pdo_mapping.kind {
            PdoType::RPDO => calculate_pdo_index_offset(
                ObjectDictionary::RPDO_MAPPING_PARAMETER_BASE_INDEX,
                mapping_number,
            ),
            PdoType::TPDO => calculate_pdo_index_offset(
                ObjectDictionary::TPDO_MAPPING_PARAMETER_BASE_INDEX,
                mapping_number,
            ),
        };
        trace!(
            "2. Deactivate the mapping by setting subindex 00h of the corresponding mapping parameter ({}) to \"0\".",
            mapping_index
        );
        let data = [0];
        self.sdo
            .lock()
            .await
            .download(mapping_index, 0x0, &data)
            .await
            .map_err(DriveError::CanOpen)?;

        trace!("3. Change the mapping in the desired subindices.");
        for (number, mapping) in pdo_mapping.mappings.iter().enumerate() {
            // Construct the payload: 2 bytes of OD entry to be mapped, 1 byte subindex, 1 byte with number of bits to be mapped
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
                .download(mapping_index, number as u8, &data)
                .await
                .map_err(DriveError::CanOpen)?;
        }

        trace!(
            "4. Activate the mapping by writing the number of objects that are to be mapped in subindex 00h of the corresponding mapping parameter (e.g., 1600h:00h)."
        );
        let data = [pdo_mapping.mappings.len() as u8];
        self.sdo
            .lock()
            .await
            .download(mapping_index, 0x0, &data)
            .await
            .map_err(DriveError::CanOpen)?;

        Ok(())
    }

    /// Parametrize this NanotecMotor
    /// parametrisation is the process of setting important parameters like maximum velocity or
    /// torque to known values at boot
    /// The motor usually does not commit these changes to nv memory, so this has to run on every
    /// new boot cycle of the device
    pub async fn parametrize(&self, parameters: &[SdoAction<'_>]) -> Result<()> {
        trace!(
            "Starting parametrisation of NanotecMotor with node id {}",
            self.node_id
        );
        // parametrisation is done through a series of SDO calls, perform these in order
        for param in parameters {
            trace!("parametrizing nodeId {}: {param:?}", self.node_id);
            match param {
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
                        .map_err(DriveError::CanOpen)?;
                }
                SdoAction::Upload { index, subindex } => {
                    self.sdo
                        .lock()
                        .await
                        .upload(*index, *subindex)
                        .await
                        .map_err(DriveError::CanOpen)?;
                }
            }
        }

        Ok(())
    }
}

fn decode_statusword(sw: u16) -> Cia402State {
    todo!()
}

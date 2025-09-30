use std::sync::Arc;

use anyhow::Result;
use oze_canopen::sdo_client::SdoClient;
use tokio::sync::Mutex;
use tracing::*;

use crate::{
    comms::pdo::mapping::{PdoMapping, PdoType},
    error::DriveError,
    od::ObjectDictionary,
};

/// Configure the devicde with the given set of PDO mappings
pub async fn configure_pdo_mappings(
    node_id: u8,
    sdo: Arc<Mutex<SdoClient>>,
    pdo_mapping: &'static [PdoMapping],
) -> Result<()> {
    trace!("configure_pdo_mappings for nodeId {}", node_id);
    for (mapping_number, mapping) in pdo_mapping.iter().enumerate() {
        set_pdo_mapping(node_id, sdo.clone(), mapping, mapping_number).await?;
    }

    Ok(())
}

/// Apply the given PDO mapping to the device
/// This follows steps listed at page 118 of PD4C_CANopen_Technical_Manual_v3.3
async fn set_pdo_mapping(
    node_id: u8,
    sdo: Arc<Mutex<SdoClient>>,
    pdo_mapping: &PdoMapping,
    mapping_number: usize,
) -> Result<()> {
    trace!(
        "Setting Pdo mapping {mapping_number}: {:?} for motor at node id {node_id}",
        pdo_mapping
    );

    let mapping_number = mapping_number as u8;

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
    sdo.lock()
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
    sdo.lock()
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
            mapping.sub_index,
            mapping.number_of_bits,
        ];
        sdo.lock()
            .await
            .download(mapping_index, number as u8, &data)
            .await
            .map_err(DriveError::CanOpen)?;
    }

    trace!(
        "4. Activate the mapping by writing the number of objects that are to be mapped in subindex 00h of the corresponding mapping parameter (e.g., 1600h:00h)."
    );
    let data = [pdo_mapping.mappings.len() as u8];
    sdo.lock()
        .await
        .download(mapping_index, 0x0, &data)
        .await
        .map_err(DriveError::CanOpen)?;

    Ok(())
}

/// Calculates pdo index offset from given base and pdo mapping number
/// For example SDO for Node Id 3 = 0x500 + 3 = 0x503
pub fn calculate_pdo_index_offset(base: u16, pdo_mapping_number: u8) -> u16 {
    base.checked_add(pdo_mapping_number.into())
        .expect("Overflow in RPDO mapping parameter index calculation")
}

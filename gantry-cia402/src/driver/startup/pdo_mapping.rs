use std::sync::Arc;

use anyhow::Result;
use oze_canopen::sdo_client::SdoClient;
use tokio::sync::Mutex;
use tracing::*;

use crate::{
    comms::{
        pdo::mapping::{PdoMapping, PdoType},
        sdo::SDO_PROCESS_DURATION,
    },
    error::DriveError,
    od::{
        RPDO_COMMUNICATION_PARAMETER_BASE_INDEX, RPDO_MAPPING_PARAMETER_BASE_INDEX,
        TPDO_COMMUNICATION_PARAMETER_BASE_INDEX, TPDO_MAPPING_PARAMETER_BASE_INDEX,
    },
};

#[derive(Debug, PartialEq)]
pub enum TransmissionType {
    OnSync,
    OnChange,
}

impl TransmissionType {
    pub fn od_value(&self) -> u8 {
        match self {
            TransmissionType::OnSync => 0x1,
            TransmissionType::OnChange => 0xFF,
        }
    }
}

/// Configure the devicde with the given set of PDO mappings
pub async fn configure_pdo_mappings(
    node_id: u8,
    sdo: Arc<Mutex<SdoClient>>,
    pdo_mapping: &'static [PdoMapping],
) -> Result<()> {
    trace!("configure_pdo_mappings for nodeId {}", node_id);
    for mapping in pdo_mapping.iter() {
        set_pdo_mapping(node_id, sdo.clone(), mapping).await?;

        tokio::time::sleep(SDO_PROCESS_DURATION).await;
    }

    Ok(())
}

/// Apply the given PDO mapping to the device
/// This follows steps listed at page 118 of PD4C_CANopen_Technical_Manual_v3.3
async fn set_pdo_mapping(
    node_id: u8,
    sdo: Arc<Mutex<SdoClient>>,
    pdo_mapping: &PdoMapping,
) -> Result<()> {
    // 1. Deactivate the PDO by setting the Valid Bit (bit 31) of subindex 01h of the corresponding communication parameter (e.g., 1400h:01h) to "1".
    let (communication_index, num) = match pdo_mapping.pdo {
        PdoType::RPDO(num) => (
            calculate_pdo_index_offset(RPDO_COMMUNICATION_PARAMETER_BASE_INDEX, num),
            num,
        ),
        PdoType::TPDO(num) => (
            calculate_pdo_index_offset(TPDO_COMMUNICATION_PARAMETER_BASE_INDEX, num),
            num,
        ),
    };
    info!(
        "Setting Pdo mapping {num}: {:?} for motor at node id {node_id}",
        pdo_mapping
    );

    let validate_bytes = sdo
        .lock()
        .await
        .upload(communication_index, 0x1)
        .await
        .map_err(DriveError::CanOpen)?;

    let validate_pdo = u32::from_le_bytes(
        validate_bytes
            .clone()
            .try_into()
            .map_err(DriveError::Conversion)?,
    );
    trace!(
        "0. Fetched current COB-ID: {:#0x} -> (node, RPDO base COB-ID): ({:#0x}, {:#0x})",
        validate_pdo,
        validate_pdo as u8,
        (validate_pdo & !(u8::MAX as u32)) as u16,
    );

    let invalidate_pdo = validate_pdo | (1 << 31);

    trace!(
        "1. Deactivate the PDO by setting the Valid Bit (bit 31) of subindex 01h of the
            corresponding PDO communication parameter ({}) to \"1\". -> Invalidation value: {:#0x}",
        communication_index, invalidate_pdo
    );
    let invalidate_data = invalidate_pdo.to_le_bytes();
    sdo.lock()
        .await
        .download(communication_index, 0x1, &invalidate_data)
        .await
        .map_err(DriveError::CanOpen)?;

    trace!(
        "1.B Set Transmission type to {:?}",
        pdo_mapping.transmission_type
    );
    sdo.lock()
        .await
        .download(
            communication_index,
            0x2,
            &[pdo_mapping.transmission_type.od_value()],
        )
        .await
        .map_err(DriveError::CanOpen)?;

    if let PdoType::TPDO(_) = pdo_mapping.pdo
        && pdo_mapping.transmission_type == TransmissionType::OnChange
    {
        // Configure a periodic event to continously synchronise the driver with the latest device
        // state
        const SYNCHRONISATION_PERIOD_MS: u16 = 500;
        const SYNCHRONISATION_SUB_IDX: u8 = 0x05;

        trace!(
            "1.C Transmission type is {:?} -> Configuring a periodic event to continously synchronise state in OD: {:#0x}:{} of val: {:x?}",
            pdo_mapping.transmission_type,
            communication_index,
            SYNCHRONISATION_SUB_IDX,
            SYNCHRONISATION_PERIOD_MS.to_le_bytes(),
        );

        sdo.lock()
            .await
            .download(
                communication_index,
                SYNCHRONISATION_SUB_IDX,
                &SYNCHRONISATION_PERIOD_MS.to_le_bytes(),
            )
            .await
            .map_err(DriveError::CanOpen)?;
    }

    // 2. Deactivate the mapping by setting subindex 00h of the corresponding mapping parameter to \"0\".,
    let mapping_index = match pdo_mapping.pdo {
        PdoType::RPDO(_) => calculate_pdo_index_offset(RPDO_MAPPING_PARAMETER_BASE_INDEX, num),
        PdoType::TPDO(_) => calculate_pdo_index_offset(TPDO_MAPPING_PARAMETER_BASE_INDEX, num),
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
    for (number, source) in pdo_mapping.sources.iter().enumerate() {
        let number = number + 1;

        trace!("3. Mapping #{number} to {source:?}");
        // Construct the payload: 2 bytes of OD entry to be mapped, 1 byte subindex, 1 byte with number of bits to be mapped
        let index_bytes = source.entry.index.to_be_bytes();
        let data: [u8; 4] = [
            index_bytes[0],
            index_bytes[1],
            source.entry.sub_index,
            source.bit_range.len,
        ];
        let vec: Vec<u8> = data.into_iter().rev().collect();

        sdo.lock()
            .await
            .download(mapping_index, number as u8, &vec)
            .await
            .map_err(DriveError::CanOpen)?;
    }

    trace!(
        "4. Activate the mapping by writing the number of objects that are to be mapped in subindex 00h of the corresponding mapping parameter (e.g., 1600h:00h)."
    );
    let data = [pdo_mapping.sources.len() as u8];
    sdo.lock()
        .await
        .download(mapping_index, 0x0, &data)
        .await
        .map_err(DriveError::CanOpen)?;

    trace!(
        "5. Activate the PDO by setting bit 31 of subindex 01h of the corresponding communication parameter (e.g., 1400h:01h) to \"0\"."
    );
    let validate_pdo = invalidate_pdo & !(1 << 31);
    sdo.lock()
        .await
        .download(communication_index, 0x1, &validate_pdo.to_le_bytes())
        .await
        .map_err(DriveError::CanOpen)?;

    Ok(())
}

/// Calculates pdo index offset from given base and pdo mapping number
/// For example SDO for Node Id 3 = 0x500 + 3 = 0x503
pub fn calculate_pdo_index_offset(base: u16, pdo_mapping_number: u8) -> u16 {
    base.checked_add((pdo_mapping_number - 1).into())
        .expect("Overflow in RPDO mapping parameter index calculation")
}

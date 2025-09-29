pub mod mapping;

use std::time::Duration;

/// PDO based Cia402Transport impl for oze-canopen
use oze_canopen::{
    interface::{CanOpenInterface, SEND_TIMOUT},
    transmitter::TxPacket,
};

use crate::{
    comms::pdo::mapping::{PdoMapping, PdoType, get_pdo_cob_id},
    error::DriveError,
    od::{
        ObjectDictionary,
        oms::{PositionSetpoint, TorqueSetpoint, VelocitySetpoint},
    },
};

/// Low level CANopen PDO transport implementation
/// Manages PDO communication to a single node_id / motor
/// Used by [`DrivePublisher::publish_updates()`]
pub struct Pdo {
    canopen: CanOpenInterface,
    node_id: u8,
    rpdo_mapping: PdoMapping,
}

impl Pdo {
    pub fn new(
        canopen: CanOpenInterface,
        node_id: u8,
        rpdo_mapping: PdoMapping,
        tpdo_mapping: PdoMapping,
    ) -> Result<Self, DriveError> {
        // Check if all required mappings are present
        Pdo::check_required_rpdo_mappings(rpdo_mapping)?;

        Self {
            canopen,
            node_id,
            rpdo_mapping,
        }
    }


    pub async fn (&self) -> Result<u16, DriveError> {

    // Write the given controlword to the motor
    pub async fn write_controlword(&self, control_word: u16) -> Result<(), DriveError> {
        // TODO this is kinda ugly, get it from the self object
        let cob_id = get_pdo_cob_id(PdoMapping::RPDO_NUM_CONTROL_WORD, PdoType::RPDO).ok_or(
            DriveError::ViolatedInvariant("Asked for the cob_id for PDO number > 4".to_string()),
        )?;

        let cw_bytes: Vec<u8> = control_word.to_be_bytes().into_iter().collect();

        self.canopen
            .tx
            .send_timeout(
                TxPacket {
                    cob_id,
                    data: cw_bytes,
                },
                Duration::from_millis(SEND_TIMOUT),
            )
            .await
            .map_err(DriveError::Timeout)?;

        Ok(())
    }

    pub async fn read_statusword(&self) -> Result<u16, DriveError> {
        todo!()
    }

    pub async fn write_position_setpoint(
        &self,
        position_setpoint: PositionSetpoint,
    ) -> Result<(), DriveError> {
        todo!()
    }

    pub async fn write_velocity_setpoint(
        &self,
        velocity_setpoint: VelocitySetpoint,
    ) -> Result<(), DriveError> {
        todo!()
    }

    pub async fn write_torque_setpoint(
        &self,
        torque_setpoint: TorqueSetpoint,
    ) -> Result<(), DriveError> {
        todo!()
    }

    // Check if these rpdo mappings contain a controlword
    // TODO: move this into type system
    fn check_required_rpdo_mappings(rpdo_mapping: PdoMapping) -> Result<(), DriveError> {
        for map in rpdo_mapping.mappings.iter() {
            if map.index == ObjectDictionary::CONTROL_WORD.index {
                return Ok(());
            }
        }
        Err(DriveError::ViolatedInvariant(
            "RPDO required mapping - control word is not present",
        ))
    }
}

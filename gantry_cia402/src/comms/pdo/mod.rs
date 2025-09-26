pub mod manager;
pub mod mapping;

use std::{thread::JoinHandle, time::Duration};

/// PDO based Cia402Transport impl for oze-canopen
use oze_canopen::{
    interface::{CanOpenInterface, SEND_TIMOUT},
    transmitter::TxPacket,
};

use crate::{
    comms::pdo::mapping::{PdoMapping, PdoType, get_pdo_cob_id},
    error::DriveError,
    od::oms::{PositionSetpoint, TorqueSetpoint, VelocitySetpoint},
};

pub struct Pdo {
    canopen: CanOpenInterface,
    node_id: u8,
    // TODO all mapping info
}

impl Pdo {
    // TODO pass in everything required for write_controlword to work reliably
    // aka a controlword should be mapped to some pdo
    // Also think about the requirements for write_position_setpoint et al
    pub fn new() -> Self {}

    pub async fn write_controlword(&self, cw: u16) -> Result<(), DriveError> {
        // TODO this is kinda ugly, get it from the self object
        let cob_id = get_pdo_cob_id(PdoMapping::RPDO_NUM_CONTROL_WORD, PdoType::RPDO).ok_or(
            DriveError::ViolatedInvariant("Asked for the cob_id for PDO number > 4".to_string()),
        )?;

        let cw_bytes: Vec<u8> = cw.to_be_bytes().into_iter().collect();

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
}

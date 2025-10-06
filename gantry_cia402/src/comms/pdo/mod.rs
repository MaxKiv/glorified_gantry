pub mod frame;
pub mod mapping;

use std::time::Duration;

/// PDO based Cia402Transport impl for oze-canopen
use oze_canopen::{
    interface::{CanOpenInterface, SEND_TIMOUT},
    transmitter::TxPacket,
};

use crate::{
    comms::pdo::{
        frame::PdoFrame,
        mapping::{PdoMapping, PdoType, get_pdo_cob_id},
    },
    driver::{
        oms::{OperationMode, PositionSetpoint, TorqueSetpoint, VelocitySetpoint},
        state::Cia402Flags,
        update::ControlWord,
    },
    error::DriveError,
    od::{ODEntry, ObjectDictionary},
};

/// Low level CANopen PDO transport implementation
/// Manages PDO communication to a single node_id / motor
/// Used by the update publisher
pub struct Pdo {
    canopen: CanOpenInterface,
    node_id: u8,
    rpdo_mapping_set: &'static [PdoMapping],
    rpdos: [PdoFrame; 4],
}

impl Pdo {
    pub fn new(
        canopen: CanOpenInterface,
        node_id: u8,
        rpdo_mapping_set: &'static [PdoMapping],
    ) -> Result<Self, DriveError> {
        // Check if all required mappings are present
        Pdo::check_required_rpdo_mappings(rpdo_mapping_set)?;

        Ok(Self {
            canopen,
            node_id,
            rpdo_mapping_set,
            rpdos: core::array::from_fn(|_| PdoFrame::zero()),
        })
    }

    // Perform the given cia402 state transition by writing the corresponding controlword flags and
    // sending the PDO that has controlword mapped out to the device
    pub async fn write_cia402_state_transition(
        &mut self,
        flags: Cia402Flags,
    ) -> Result<(), DriveError> {
        // Set the cia402 controlword bits to represent the requested state
        let cw = self.get_current_controlword();
        cw.with_cia402_flags(flags);
        self.set_controlword(cw);

        self.send_rpdo(PdoMapping::RPDO_IDX_CONTROL_WORD).await?;

        Ok(())
    }

    // TODO: cleanup all the hardcoded addresses and offsets when you have time... I will have time
    // for that, right?
    pub async fn write_position_setpoint(
        &mut self,
        PositionSetpoint {
            flags,
            target,
            profile_velocity,
        }: PositionSetpoint,
    ) -> Result<(), DriveError> {
        // 1. Construct RPDO1: Set opmode to position and toggle control_word OMS bits

        // Set Controlword
        let cw = self.get_current_controlword();
        cw.with_position_flags(flags);
        self.set_controlword(cw);

        // Set Position Mode
        self.set_operational_mode(OperationMode::ProfilePosition);

        // Send RPDO1
        self.send_rpdo(PdoMapping::RPDO_IDX_OPMODE).await?;

        // 2. Construct RPDO2: Set position and velocity target
        self.rpdos[PdoMapping::RPDO_IDX_TARGET_POS].set(
            PdoMapping::POS_TARGET_OFFSET,
            &(target as u32).to_be_bytes(),
        );
        self.rpdos[PdoMapping::RPDO_IDX_TARGET_POS].set(
            PdoMapping::POS_VEL_OFFSET,
            &(profile_velocity.to_be_bytes()),
        );

        // Send RPDO2
        self.send_rpdo(PdoMapping::RPDO_IDX_TARGET_POS).await?;

        Ok(())
    }

    pub async fn write_velocity_setpoint(
        &mut self,
        VelocitySetpoint {
            target_velocity: target,
        }: VelocitySetpoint,
    ) -> Result<(), DriveError> {
        // Set Velocity Mode
        self.set_operational_mode(OperationMode::ProfileVelocity);

        self.send_rpdo(PdoMapping::RPDO_IDX_OPMODE).await?;

        // Set position and velocity target
        self.rpdos[PdoMapping::RPDO_IDX_TARGET_VEL]
            .set(PdoMapping::VEL_TARGET_OFFSET, &target.to_be_bytes());

        self.send_rpdo(PdoMapping::RPDO_IDX_TARGET_VEL).await?;

        Ok(())
    }

    pub async fn write_torque_setpoint(
        &mut self,
        TorqueSetpoint {
            target_torque: target,
        }: TorqueSetpoint,
    ) -> Result<(), DriveError> {
        // Set Torque Mode
        self.set_operational_mode(OperationMode::ProfileTorque);

        self.send_rpdo(PdoMapping::RPDO_IDX_OPMODE).await?;

        // Set position and torque target
        self.rpdos[PdoMapping::RPDO_IDX_TARGET_TORQUE]
            .set(PdoMapping::VEL_TARGET_OFFSET, &target.to_be_bytes());

        self.send_rpdo(PdoMapping::RPDO_IDX_TARGET_TORQUE).await?;

        Ok(())
    }

    // Check if these rpdo mappings contain a controlword
    // TODO: move this into type system
    fn check_required_rpdo_mappings(
        rpdo_mapping_set: &'static [PdoMapping],
    ) -> Result<(), DriveError> {
        if Self::check_if_mapped(rpdo_mapping_set, &ObjectDictionary::CONTROL_WORD)
            && Self::check_if_mapped(rpdo_mapping_set, &ObjectDictionary::SET_OPERATION_MODE)
        {
            Ok(())
        } else {
            Err(DriveError::ViolatedInvariant(
                format!(
                    "RPDO mapping check failed for {rpdo_mapping_set:?} - control word is not present"
                )
                .to_string(),
            ))
        }
    }

    fn check_if_mapped(rpdo_mapping_set: &'static [PdoMapping], entry: &ODEntry) -> bool {
        for rpdo in rpdo_mapping_set {
            for map in rpdo.mappings {
                if map.index == entry.index {
                    return true;
                }
            }
        }
        false
    }

    /// Gets current control word
    fn get_current_controlword(&self) -> ControlWord {
        let cw_bytes = [
            self.rpdos[PdoMapping::RPDO_IDX_CONTROL_WORD].data[0],
            self.rpdos[PdoMapping::RPDO_IDX_CONTROL_WORD].data[1],
        ];
        ControlWord::from_bits(u16::from_be_bytes(cw_bytes)).expect(
            "unable to fetch current controlword from saved RPDO1 in write_position_setpoint",
        )
    }

    async fn send_rpdo(&mut self, rpdo_num: usize) -> Result<(), DriveError> {
        let cob_id =
            get_pdo_cob_id(rpdo_num, PdoType::RPDO).ok_or(DriveError::ViolatedInvariant(
                "Asked for the cob_id for PDO number: {rpdo_num} > 4".to_string(),
            ))?;

        self.canopen
            .tx
            .send_timeout(
                TxPacket {
                    cob_id,
                    data: self.rpdos[rpdo_num].data.to_vec(),
                },
                Duration::from_millis(SEND_TIMOUT),
            )
            .await
            .map_err(DriveError::Timeout)?;

        Ok(())
    }

    fn set_controlword(&mut self, cw: ControlWord) {
        self.rpdos[PdoMapping::RPDO_IDX_CONTROL_WORD]
            .set(PdoMapping::CONTROL_WORD_OFFSET, &cw.bits().to_be_bytes());
    }

    fn set_operational_mode(&mut self, mode: OperationMode) {
        self.rpdos[PdoMapping::RPDO_IDX_OPMODE].set(PdoMapping::OPMODE_OFFSET, &[mode as u8]);
    }
}

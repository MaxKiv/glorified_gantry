use crate::{drive::CiA402Drive, error::DriveError};

impl CiA402Drive {
    // Generic
    pub async fn halt(&mut self) -> Result<(), DriveError> {
        todo!()
    }

    // Position
    pub async fn move_to(&mut self, abs_pos: i32) -> Result<(), DriveError> {
        todo!()
    }
    pub async fn move_by(&mut self, delta: i32) -> Result<(), DriveError> {
        todo!()
    }

    // Velocity
    pub async fn set_velocity(&mut self, vel: i32) -> Result<(), DriveError> {
        todo!()
    }

    // Torque
    pub async fn set_torque(&mut self, tq: i16) -> Result<(), DriveError> {
        todo!()
    }
}

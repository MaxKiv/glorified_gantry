use crate::{drive::CiA402Drive, error::DriveError};

impl CiA402Drive {
    pub async fn get_status(&self) -> Result<Cia402Status, DriveError> {
        todo!()
    }
    pub async fn get_position(&self) -> Result<i32, DriveError> {
        todo!()
    }
    pub async fn get_velocity(&self) -> Result<i32, DriveError> {
        todo!()
    }
    pub async fn get_torque(&self) -> Result<i16, DriveError> {
        todo!()
    }
}

use crate::{drive::CiA402Drive, error::DriveError};

impl CiA402Drive {
    pub async fn shutdown(&mut self) -> Result<(), DriveError> {
        todo!()
    }

    // Switch On → Operation Enabled
    pub async fn enable(&mut self) -> Result<(), DriveError> {
        todo!()
    }

    pub async fn disable(&mut self) -> Result<(), DriveError> {
        todo!()
    }
}

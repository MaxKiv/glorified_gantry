pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    use gantry_axis::sync::SyncMaster;
    use gantry_cia402::driver::{
        builder::Cia402DriverBuilder,
        event::MotorEvent,
        identifier::{Cia402Identifier, CiaProfileNumber},
        receiver::subscriber::wait_for_event,
        state::Cia402State,
    };
    use gantry_demo::config::{DeviceName, TEST_SETUP_MOTOR_TYPE};

    use crate::common::COMMS_TIMEOUT;

    use super::*;

    pub const INVALID_DEVICE_NAME: DeviceName = "BadName";
    pub const INVALID_IDENTIFIER: Cia402Identifier = Cia402Identifier {
        node_id: 1,
        device_profile_number: CiaProfileNumber::Bad,
        motor_type: TEST_SETUP_MOTOR_TYPE,
        device_name: INVALID_DEVICE_NAME,
    };

    #[tokio::test]
    async fn test_invalid_identifier() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        let identifier = INVALID_IDENTIFIER;
        let node_id = identifier.node_id;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        info!("Initializing Cia402Driver for motor driver at node id {node_id}");
        let sync_master = SyncMaster::init(canopen.clone());
        let sync_rx = sync_master.get_sync_receiver();

        info!("Initializing Cia402Driver for motor driver at node id {node_id}");
        let init_result = Cia402DriverBuilder::new(identifier)
            .with_canopen(canopen.clone())
            .with_default_pdo_mappings()
            .with_default_parameters()
            .with_sync_receiver(sync_rx)
            .build()
            .await;

        assert!(
            init_result.is_err(),
            "Initialisation with bad identifier should error, but did not"
        );

        Ok(())
    }
}

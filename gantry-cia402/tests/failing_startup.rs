pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    use gantry_axis::sync::SyncMaster;
    use gantry_cia402::{
        comms::sdo::SdoAction,
        driver::builder::Cia402DriverBuilder,
        od::{access::AccessType, entry::ODEntry, mappable::MappableType, value::ODValue},
    };

    

    use super::*;

    /// Target position [counts] (default 3600 counts = 1 rev)
    pub const INVALID_ODENTRY: ODEntry = ODEntry::new(
        0xAAAA,
        0x00,
        AccessType::ReadWrite,
        MappableType::TPDO,
        ODValue::I32(0x0000_0FA0),
    );

    pub const INVALID_PARAMS: &[SdoAction] = &[
        // --- Profile Position ---
        // Set target position = 0 (we start from home or zero)
        SdoAction::Download {
            entry: &INVALID_ODENTRY,
            data: &69i32.to_le_bytes(),
        },
    ];

    #[tokio::test]
    async fn test_startup_failure() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        let identifier = common::TEST_MOTOR;
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
            .with_parameters(INVALID_PARAMS)
            .with_sync_receiver(sync_rx)
            .build()
            .await;

        assert!(
            init_result.is_err(),
            "Initialisation with Invalid Parameters should error, but it didnt"
        );

        Ok(())
    }
}

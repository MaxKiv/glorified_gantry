pub mod common;


use tracing::*;

#[cfg(test)]
mod tests {

    use gantry_axis::sync::SyncMaster;
    use gantry_cia402::driver::{
            builder::Cia402DriverBuilder, event::MotorEvent,
            receiver::subscriber::wait_for_event, state::Cia402State,
        };

    use crate::common::{COMMS_TIMEOUT, PARAMS};

    use super::*;

    #[tokio::test]
    async fn test_startup() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        let identifier = common::TEST_MOTOR;
        let node_id = identifier.node_id;

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

        info!("Initializing Cia402Driver for motor driver at node id {node_id}");
        let sync_master = SyncMaster::init(canopen.clone());
        let sync_rx = sync_master.get_sync_receiver();

        info!("Initializing Cia402Driver for motor driver at node id {node_id}");
        let drive = Cia402DriverBuilder::new(identifier)
            .with_canopen(canopen.clone())
            .with_default_pdo_mappings()
            .with_parameters(PARAMS)
            .with_sync_receiver(sync_rx)
            .build()
            .await?;

        info!("Wait for Cia402State::OperationEnabled");
        wait_for_event(
            drive.event_rx.resubscribe(),
            MotorEvent::Cia402StateUpdate(Cia402State::OperationEnabled),
            COMMS_TIMEOUT,
        )
        .await?;

        Ok(())
    }
}

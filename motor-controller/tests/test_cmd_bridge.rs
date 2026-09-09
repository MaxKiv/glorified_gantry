pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    use motor_controller::{
        frontend::{GantryCommand, GantryFrontend},
        rt::engine::RtEngine,
    };

    use crate::common::setup_tracing_subscriber;

    use super::*;

    #[test]
    fn test_command_bridge() -> anyhow::Result<()> {
        setup_tracing_subscriber();

        let (rt_cmd_tx, rt_cmd_rx) = CmdChannel::<CMD_CHANNEL_SIZE>::new()?;

        info!("Starting RT engine");
        let rt_engine = RtEngine::start(String::from("can0"), rt_cmd_rx);

        info!("Starting tokio reactor");
        let tokio = std::thread::spawn(move || {
            let tokio_rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            info!("Starting Non-RT Frontend");
            let frontend = GantryFrontend::start_on(tokio_rt.handle(), rt_cmd_tx);

            tokio_rt.block_on(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                    info!("tokio sending shutdown");
                    frontend.send(GantryCommand::Home)?;
                }
            });
        });

        if let Err(err) = rt_engine.join() {
            anyhow::bail!("failed to join rt engine thread: {err:?}");
        }
        if let Err(err) = tokio.abort() {
            anyhow::bail!("failed to join tokio reactor thread: {err:?}");
        }

        Ok(())
    }
}

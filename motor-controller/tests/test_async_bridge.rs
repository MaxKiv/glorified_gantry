pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    use motor_controller::{
        frontend::{CMD_CH_SIZE, EVENT_CH_SIZE, GantryFrontend, cmd::GantryCommand},
        rt::{cmd::channel::CmdChannel, engine::RtEngine, event::event_channel},
    };

    use crate::common::setup_tracing_subscriber;

    use super::*;

    #[test]
    fn test_async_bridge() -> anyhow::Result<()> {
        setup_tracing_subscriber();

        let (rt_cmd_tx, rt_cmd_rx) = CmdChannel::<CMD_CH_SIZE>::new()?;
        let (rt_event_tx, rt_event_rx) = event_channel(EVENT_CH_SIZE)?;

        info!("Starting RT engine");
        let rt_engine = RtEngine::start(String::from("can0"), rt_cmd_rx);

        info!("Starting tokio reactor");
        let tokio = std::thread::spawn(move || {
            let tokio_rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            info!("Starting Non-RT Frontend");
            let frontend = GantryFrontend::start_on(tokio_rt.handle(), rt_cmd_tx, rt_event_rx);

            tokio_rt.block_on(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                    info!("tokio sending cmd");
                    if let Err(e) = frontend.send(GantryCommand::Home).await {
                        tracing::error!("Error sending cmd from tokio: {:?}", e);
                    }
                }
            });
        });

        if let Err(err) = rt_engine.join() {
            anyhow::bail!("failed to join rt engine thread: {err:?}");
        }
        if let Err(err) = tokio.join() {
            anyhow::bail!("failed to join tokio reactor thread: {err:?}");
        }

        Ok(())
    }
}

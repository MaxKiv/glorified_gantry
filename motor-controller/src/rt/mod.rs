pub mod cmd;
pub mod engine;
pub mod error;
pub mod event;
pub mod eventfd;
pub mod feedback;
pub mod timekeeper;
pub mod timerfd;

#[cfg(test)]
mod tests {
    use crate::{
        consts::{RT_CONFIG, cfg::TEST_GANTRY_CFG},
        frontend::cmd::GantryCommand,
        rt::{
            cmd::{RtCommand, channel::CmdChannel},
            engine::RtEngine,
            event::event_channel,
        },
        utils::setup_tracing_subscriber,
    };

    use super::*;

    pub const CMD_CH_SIZE: usize = RT_CONFIG.cmd_channel_size;
    pub const EVENT_CH_SIZE: usize = RT_CONFIG.event_channel_size;

    #[test]
    fn full() -> anyhow::Result<()> {
        setup_tracing_subscriber();

        let gantry_cfg = &TEST_GANTRY_CFG;

        let (rt_cmd_tx, rt_cmd_rx) = CmdChannel::<CMD_CH_SIZE>::new()?;
        let (rt_event_tx, rt_event_rx) = event_channel(EVENT_CH_SIZE)?;

        info!("Starting RT engine");
        let rt_engine = RtEngine::start(
            String::from("can0"),
            rt_cmd_rx,
            rt_event_tx,
            gantry_cfg.clone(),
        );

        info!("Starting tokio reactor");
        let tokio = std::thread::spawn(move || {
            let tokio_rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            tokio_rt.block_on(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                    info!("tokio sending gantry cmd");
                    rt_cmd_tx
                        .send(&GantryCommand::Idle)
                        .expect("failed to notify RT");
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

pub mod cmd;
pub mod engine;
pub mod event;
pub mod eventfd;
pub mod timekeeper;
pub mod timerfd;

use crate::{
    canopen::{
        CanOpenError,
        od::entry::ODEntry,
        pdo::{PdoType, TransmissionType},
    },
    cw::ControlWord,
    oms::OperationMode,
    rt::timekeeper::CycleTiming,
    sw::StatusWord,
};
use tracing::{debug, error, info, warn};

#[derive(Clone, Copy, Debug)]
pub struct RtSetpoint {
    generation: u64,
    controlword: ControlWord,
    target: i32,
    // OR?
    // target: MotorSetpoint(ProfilePosition(10))
}

#[derive(Debug, thiserror::Error)]
pub enum RtError {
    #[error("Timer error")]
    Timer,
    #[error("Polling error")]
    Poll,
    #[error("Invalid Motor")]
    InvalidMotor,
    #[error("Unable to start motor")]
    Startup,
    #[error("Canopen Error: {0:?}")]
    CanOpen(CanOpenError),
}

pub struct MotorFeedback {
    pub sw: StatusWord,
    pub pos: i32,
    pub vel: i32,
    pub torque: i16,
    pub opmode: OperationMode,
}

impl MotorFeedback {
    pub fn new() -> Self {
        Self {
            sw: StatusWord::default(),
            pos: 0,
            vel: 0,
            torque: 0,
            opmode: OperationMode::default(),
        }
    }
}

struct RtFeedback<const N: usize> {
    cycle: u64,
    motors: [MotorFeedback; N],
    timing: CycleTiming,
    errors: RtError,
    skew: Option<f64>,
}

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

pub mod cmd;
pub mod engine;
pub mod eventfd;
pub mod timekeeper;
pub mod timerfd;

use crate::{
    canopen::{
        od::entry::ODEntry,
        pdo::{PdoType, TransmissionType},
    },
    cw::ControlWord,
    oms::OperationMode,
    rt::timekeeper::CycleTiming,
    sw::StatusWord,
};
use tracing::{debug, error, info, warn};

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
}

pub struct MotorFeedback {
    pub sw: StatusWord,
    pub pos: i32,
    pub vel: i32,
    pub torque: i16,
    pub opmode: OperationMode,
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
        consts::RT_CONFIG,
        rt::{
            cmd::{RtCommand, channel::CmdChannel},
            engine::RtEngine,
        },
        utils::setup_tracing_subscriber,
    };

    use super::*;

    const CMD_CHANNEL_SIZE: usize = RT_CONFIG.cmd_channel_size;

    #[test]
    fn rt() -> anyhow::Result<()> {
        setup_tracing_subscriber();

        let (cmd_tx, cmd_rx) = CmdChannel::<CMD_CHANNEL_SIZE>::new()?;

        info!("Starting rt engine");
        let rt_engine = RtEngine::start(String::from("can0"), cmd_rx);

        info!("Starting tokio reactor");
        let tokio = std::thread::spawn(move || {
            let tokio_rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            tokio_rt.block_on(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                    info!("tokio sending shutdown");
                    cmd_tx
                        .send(RtCommand::SingleCycle)
                        .expect("failed to notify RT");

                    // tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    // cmd_tx
                    //     .send(RtCommand::Reconfigure)
                    //     .expect("failed to notify RT");
                    // cmd_tx
                    //     .send(RtCommand::Shutdown)
                    //     .expect("failed to notify RT");
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

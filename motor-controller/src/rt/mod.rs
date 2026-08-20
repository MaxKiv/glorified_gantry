pub mod cmd;
pub mod engine;
pub mod timekeeper;
pub mod timerfd;

use crate::{cw::ControlWord, rt::timekeeper::CycleTiming};
use tracing::{debug, error, info, warn};

pub struct RtSetpoint {
    generation: u64,
    controlword: ControlWord,
    target: i32,
    // OR?
    // target: MotorSetpoint(ProfilePosition(10))
}

pub struct RtConfig {
    // pdo_mapping: PdoMapping,
}

pub struct RtTimeComms {
    // event_rx: RT_API::Receiver<RealTimeFeedback>,
}

#[derive(Debug, thiserror::Error)]
pub enum RtErrors {
    #[error("other")]
    Other,
}

pub struct MotorFeedback {}

struct RtFeedback<const N: usize> {
    cycle: u64,
    motors: [MotorFeedback; N],
    timing: CycleTiming,
    errors: RtErrors,
    skew: Option<f64>,
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    use crate::rt::{
        cmd::{RtCommand, channel::CmdChannel},
        engine::RtEngine,
    };

    use super::*;

    #[test]
    fn rt() -> anyhow::Result<()> {
        setup_tracing_subscriber();

        let (cmd_tx, cmd_rx) = CmdChannel::<8>::new()?;

        info!("Starting rt engine");
        let rt_engine = RtEngine::start(String::from("can0"), cmd_rx);

        let tokio_rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        tokio_rt.block_on(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                info!("tokio sending shutdown");
                cmd_tx
                    .send(RtCommand::Shutdown)
                    .expect("failed to notify RT");

                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                cmd_tx.send(RtCommand::UrMom).expect("failed to notify RT");
                cmd_tx.send(RtCommand::UrMom).expect("failed to notify RT");
            }
        });

        info!("Joining rt engine");
        if let Err(err) = rt_engine.join() {
            anyhow::bail!("failed to join rt engine thread: {err:?}");
        }

        Ok(())
    }

    fn setup_tracing_subscriber() {
        // a builder for `FmtSubscriber`.
        let subscriber = FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_max_level(Level::DEBUG)
            // completes the builder.
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }
}

pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    use motor_controller::{
        frontend::{
            CMD_CH_SIZE, EVENT_CH_SIZE, GantryFrontend,
            cmd::{GantryCommand, GantrySetpoint},
            event::GantryEvent,
        },
        oms::{
            position::{PositionFlagsCW, PositionFlagsSW, PositionSetpoint},
            setpoint::Setpoint,
            torque::TorqueSetpoint,
        },
        rt::{cmd::channel::CmdChannel, engine::RtEngine, event::event_channel},
    };
    use tokio::sync::{Barrier, broadcast::error::TryRecvError};

    use crate::common::setup_tracing_subscriber;

    use super::*;

    pub const TEST_EVENTS: &[GantryEvent] = &[
        GantryEvent::Configuring,
        GantryEvent::Operational,
        GantryEvent::StateFeedback,
        GantryEvent::Error,
    ];

    #[test]
    fn test_event_bridge() -> anyhow::Result<()> {
        setup_tracing_subscriber();

        let (rt_cmd_tx, rt_cmd_rx) = CmdChannel::<CMD_CH_SIZE>::new()?;
        let (mut rt_event_tx, rt_event_rx) = event_channel(EVENT_CH_SIZE)?;

        // info!("Starting RT engine");
        // let rt_engine = RtEngine::start(String::from("can0"), rt_cmd_rx, rt_event_tx);

        info!("Starting tokio reactor");

        let sender = std::thread::spawn(move || {
            // jam events in channel
            for event in TEST_EVENTS.iter().cloned() {
                info!("Sending event: {:?}", event);
                rt_event_tx.push(event);
            }
            // notify async
            info!("Notify async");
            rt_event_tx.notify();
        });

        let tokio = std::thread::spawn(move || -> anyhow::Result<()> {
            let tokio_rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;

            info!("Starting Non-RT Frontend");

            let barrier = Barrier::new(1);
            let frontend = GantryFrontend::start_on(tokio_rt.handle(), rt_cmd_tx, rt_event_rx);
            let mut recv = frontend.event_rx.resubscribe();

            tokio_rt.block_on(async {
                info!("tokio waiting for events");
                barrier.wait().await;

                for (i, expected) in TEST_EVENTS.iter().enumerate() {
                    let event = recv.recv().await?;

                    assert_eq!(
                        &event, expected,
                        "#{} recv {:?} != send {:?}",
                        i, event, expected
                    );

                    debug!("#{} recv {:?} == send {:?}", i, event, expected);
                }

                let out = recv.try_recv();
                debug!("testing event rx is empty now: {:?}", out);

                assert!(
                    out.is_err_and(|e| e == TryRecvError::Empty),
                    "async event rx not empty"
                );

                anyhow::Ok(())
            })
        });
        if let Err(err) = sender.join() {
            anyhow::bail!("failed to join event sender thread: {err:?}");
        }
        if let Err(err) = tokio.join() {
            anyhow::bail!("failed to join tokio reactor thread: {err:?}");
        }

        Ok(())
    }
}

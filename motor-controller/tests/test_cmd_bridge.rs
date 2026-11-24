pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {

    use motor_controller::{
        frontend::{
            CMD_CH_SIZE, EVENT_CH_SIZE, GantryFrontend,
            cmd::{GantryCommand, GantrySetpoint},
        },
        oms::{
            position::{PositionFlagsCW, PositionFlagsSW, PositionSetpoint},
            setpoint::Setpoint,
            torque::TorqueSetpoint,
        },
        rt::{cmd::channel::CmdChannel, engine::RtEngine, event::event_channel},
    };

    use crate::common::setup_tracing_subscriber;

    use super::*;

    pub const TEST_CMDS: &[GantryCommand] = &[
        GantryCommand::Idle,
        GantryCommand::Home,
        GantryCommand::Setpoint(GantrySetpoint {
            x: Some(Setpoint::ProfilePosition(PositionSetpoint {
                target: 1,
                flags: PositionFlagsCW::empty(),
                profile_velocity: 1,
            })),
            y: Some(Setpoint::ProfilePosition(PositionSetpoint {
                target: 2,
                flags: PositionFlagsCW::empty(),
                profile_velocity: 2,
            })),
            z: Some(Setpoint::ProfilePosition(PositionSetpoint {
                target: 3,
                flags: PositionFlagsCW::empty(),
                profile_velocity: 3,
            })),
        }),
        GantryCommand::CyclicSetpoint(GantrySetpoint {
            x: Some(Setpoint::ProfileTorque(TorqueSetpoint { target_torque: 1 })),
            y: Some(Setpoint::ProfileTorque(TorqueSetpoint { target_torque: 2 })),
            z: Some(Setpoint::ProfileTorque(TorqueSetpoint { target_torque: 3 })),
        }),
    ];

    #[test]
    fn test_cmd_bridge() -> anyhow::Result<()> {
        setup_tracing_subscriber();

        let (rt_cmd_tx, mut rt_cmd_rx) = CmdChannel::<CMD_CH_SIZE>::new()?;
        let (rt_event_tx, rt_event_rx) = event_channel(EVENT_CH_SIZE)?;

        // info!("Starting RT engine");
        // let rt_engine = RtEngine::start(String::from("can0"), rt_cmd_rx, rt_event_tx);

        info!("Starting tokio reactor");
        let tokio = std::thread::spawn(move || {
            let tokio_rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            info!("Starting Non-RT Frontend");
            let frontend = GantryFrontend::start_on(tokio_rt.handle(), rt_cmd_tx, rt_event_rx);

            tokio_rt.block_on(async move {
                for cmd in TEST_CMDS.iter().cloned() {
                    info!("tokio sending cmd: {:?}", cmd);
                    if let Err(e) = frontend.send(cmd).await {
                        tracing::error!("Error sending cmd from tokio: {:?}", e);
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                }
            });
        });

        if let Err(err) = tokio.join() {
            anyhow::bail!("failed to join tokio reactor thread: {err:?}");
        }

        // Check if cmds go through async -> rt channel in the same order
        {
            let drain = rt_cmd_rx.drain()?;
            for (num, cmd) in drain.enumerate() {
                let should_be = TEST_CMDS[num].clone();
                assert_eq!(cmd, should_be, "{:?} should be {:?}", cmd, should_be);
                info!("read {:?} = expected {:?}", cmd, should_be);
            }
        }

        // Check if all cmds are read
        assert!(
            rt_cmd_rx.drain().is_err(),
            "Cmd Channel still contains a cmd after draining"
        );

        Ok(())
    }
}

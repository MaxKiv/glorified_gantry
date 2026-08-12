pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {
    
    use gantry_demo::config::TEST_CONFIG;
    use std::time::Duration;
    use tokio::time;
    use uom::si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::millimeter_per_second,
    };

    use gantry_axis::{
        axis::setpoint::{
            AxisSetpoint::{self, AbsolutePosition, RelativePosition},
            PositionSetpoint,
        },
        command::GantryCommand,
        gantry::Gantry,
    };

    use crate::common::{SHUTDOWN_TIMEOUT, test_gantry_cmds};

    use super::*;

    #[tokio::test]
    async fn gantry_sigint_shutdown_during_command() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let cfg = TEST_CONFIG;
        let gantry = Gantry::start(canopen, cfg).await?;

        info!("Gantry constructed, waiting for SIGINT");
        // Create a shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        // Spawn SIGINT signal handler
        tokio::spawn(async {
            tokio::signal::ctrl_c().await.unwrap();
            // Shut down gantry
            if let Err(_) = shutdown_tx.send(true) {
                panic!("Unable to send shutdown signal")
            }
        });

        // Spawn task that SIGINTs the main test process after a while
        let pid = unsafe { libc::getpid() };
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(10)).await;

            unsafe {
                libc::kill(pid, libc::SIGINT);
            }
        });

        let cmds = vec![
            GantryCommand::Home,
            GantryCommand::Setpoint {
                x: Some(AxisSetpoint::Velocity(
                    gantry_axis::axis::setpoint::VelocitySetpoint {
                        target: Velocity::new::<uom::si::velocity::millimeter_per_second>(1.0),
                    },
                )),
                y: None,
                z: None,
            },
            GantryCommand::Setpoint {
                x: None,
                y: None,
                z: Some(AbsolutePosition(PositionSetpoint {
                    target: Length::new::<millimeter>(10.0),
                    velocity: Velocity::new::<millimeter_per_second>(1.0),
                })),
            },
            GantryCommand::Setpoint {
                x: None,
                y: None,
                z: Some(RelativePosition(PositionSetpoint {
                    target: Length::new::<millimeter>(20.0),
                    velocity: Velocity::new::<millimeter_per_second>(0.01),
                })),
            },
        ];

        let timeout = Duration::from_secs(12);
        tokio::select! {
            // Execute commands
            _ = test_gantry_cmds(&gantry, &cmds, "Sigint_During_Command", timeout)=> {
                anyhow::bail!("Gantry test commands should have been interrupted by SIGINT but
                    instead ran to completion")
            },
            // Wait for SIGINT
            _ = shutdown_rx => {
                info!("Shutdown signal received, shutting down gantry");
                // Check if gantry shutdown happens in time
                return match time::timeout(SHUTDOWN_TIMEOUT, gantry.wait_for_shutdown()).await {
                    Ok(_) => Ok(()),
                    Err(_) => anyhow::bail!("Shutdown timeout ({:?}) exceeded", SHUTDOWN_TIMEOUT,),
                };
            }
        }
    }
}

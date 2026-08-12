pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {
    
    use gantry_demo::config::TEST_CONFIG;
    use std::time::Duration;
    use tokio::time;

    use gantry_axis::gantry::Gantry;

    use crate::common::SHUTDOWN_TIMEOUT;

    use super::*;

    #[tokio::test]
    async fn gantry_handle_sigint() -> anyhow::Result<()> {
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
            tokio::time::sleep(Duration::from_millis(1000)).await;

            unsafe {
                libc::kill(pid, libc::SIGINT);
            }
        });

        // Wait for SIGINT
        let _ = shutdown_rx.await;
        info!("Shutdown signal received, shutting down gantry");

        // Check if gantry shutdown happens in time
        return match time::timeout(SHUTDOWN_TIMEOUT, gantry.wait_for_shutdown()).await {
            Ok(_) => Ok(()),
            Err(_) => anyhow::bail!("Shutdown timeout ({:?}) exceeded", SHUTDOWN_TIMEOUT,),
        };
    }
}

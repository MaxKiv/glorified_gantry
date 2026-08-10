pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use gantry_demo::config::TEST_CONFIG;
    use std::time::Duration;

    use gantry_axis::{command::GantryCommand, gantry::Gantry};

    use super::*;

    #[tokio::test]
    async fn gantry_shutdown() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let cfg = TEST_CONFIG;
        let mut gantry = Gantry::start(canopen, cfg).await?;

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Shut down gantry
        gantry.shutdown();

        // Attempt to home gantry
        let cmd = GantryCommand::Home;
        gantry.send_command(cmd).await?;
        let timeout = Duration::from_secs(10);
        let out = gantry_axis::event::util::wait_until_cmd_completed(
            GantryCommand::Home,
            gantry.get_event_rx(),
            &gantry,
            timeout,
        )
        .await
        .context("Timed out waiting for {cmd}");

        assert!(
            out.is_err(),
            "Gantry should not be able to Home after shutting down, but it did anyway",
        );

        Ok(())
    }
}

pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use std::time::Duration;

    use gantry_axis::{command::GantryCommand, gantry::Gantry};

    use super::*;

    #[tokio::test]
    async fn home_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let cfg = gantry_demo::config::TEST_CONFIG;
        let gantry = Gantry::start(canopen, cfg).await?;

        let cmds = vec![
            // Home first
            GantryCommand::Home,
        ];

        let cmd = GantryCommand::Home;
        gantry.send_command(cmd).await?;
        let timeout = Duration::from_secs(10);
        gantry_axis::event::util::wait_until_cmd_completed(
            GantryCommand::Home,
            gantry.get_event_rx(),
            &gantry,
            timeout,
        )
        .await
        .context("Timed out waiting for {cmd}")?;

        // test_gantry_cmds(gantry, &cmds, "Homing", timeout).await?;

        Ok(())
    }
}

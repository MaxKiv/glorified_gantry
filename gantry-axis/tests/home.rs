pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use gantry_axis::{command::GantryCommand, gantry::Gantry};
    use gantry_cia402::driver::receiver::subscriber::wait_for_homing_completed;
    use gantry_demo::config::TEST_CONFIG;
    use tokio::signal;

    use crate::common::test_gantry_cmds;

    use super::*;

    #[tokio::test]
    async fn home_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let cfg = TEST_CONFIG;
        let gantry = Gantry::start(canopen, cfg).await?;

        let cmds = vec![
            // Home first
            GantryCommand::Home,
        ];

        let timeout = Duration::from_secs(10);
        gantry.send_command(GantryCommand::Home).await?;
        test_gantry_cmds(gantry, &cmds, "Homing", timeout).await?;

        Ok(())
    }
}

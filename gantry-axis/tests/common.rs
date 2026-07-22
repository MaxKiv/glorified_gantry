use gantry_axis::{
    command::GantryCommand, event::util::wait_until_gantry_command_completed, gantry::Gantry,
};
use tokio::time::Duration;
use tracing::*;

pub const TIMEOUT: Duration = Duration::from_secs(5);

pub async fn test_gantry_cmds(
    gantry: Gantry,
    cmds: &[GantryCommand],
    name: &str,
) -> anyhow::Result<()> {
    info!("Starting Gantry test {} with commands: {:?}", name, cmds);

    for (num, cmd) in cmds.iter().enumerate() {
        info!("{} - sending {:?}", num, cmds);
        if let Err(err) = wait_until_gantry_command_completed(
            cmd.clone(),
            gantry.get_event_rx(),
            &gantry,
            TIMEOUT,
        )
        .await
        {
            error!("ERROR in gantry test {} -> {:?}", name, err);
        }
    }

    info!("Gantry test {} completed succesfully", name);

    Ok(())
}

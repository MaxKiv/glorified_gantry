use gantry_axis::{
    command::GantryCommand,
    event::util::send_cmd_and_wait_until_gantry_command_completed,
    gantry::Gantry,
};
use tokio::time::Duration;
use tracing::*;

pub const TIMEOUT: Duration = Duration::from_secs(5);
pub const SHUTDOWN_TIMEOUT: Duration = Duration::from_millis(600);

pub async fn test_gantry_cmds(
    gantry: &Gantry,
    cmds: &[GantryCommand],
    name: &str,
    timeout: Duration,
) -> anyhow::Result<()> {
    info!("Starting Gantry test {} with commands: {:?}", name, cmds);

    for (num, cmd) in cmds.iter().enumerate() {
        info!("{} - sending {:?}", num, cmd);
        if let Err(err) = send_cmd_and_wait_until_gantry_command_completed(
            cmd.clone(),
            gantry.get_event_rx(),
            &gantry,
            timeout,
        )
        .await
        {
            error!("ERROR in gantry test {} -> {:?}", name, err);
            return Err(err);
        }
    }

    info!("Gantry test {} completed succesfully", name);

    Ok(())
}

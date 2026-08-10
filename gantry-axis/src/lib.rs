use tokio::task::{AbortHandle, JoinHandle, JoinSet};
use tracing::*;

pub mod axis;
pub mod axis_state;
pub mod cfg;
pub mod command;
pub mod diagnostic;
pub mod event;
pub mod gantry;
pub mod setpoint;
pub mod sync;

pub type OperationMode = gantry_cia402::driver::oms::OperationMode;

/// Helper that spawns a task and logs error if it ever exits
pub fn spawn_logged<F>(name: &'static str, fut: F) -> JoinHandle<()>
where
    F: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    tokio::spawn(async move {
        if let Err(e) = fut.await {
            error!("{name} task failed: {e:?}");
        }
    })
}

/// Helper that spawns a task and logs error if it ever exits
pub fn spawn_logged_joinset<F>(set: &mut JoinSet<()>, name: &'static str, fut: F) -> AbortHandle
where
    F: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    set.spawn(async move {
        if let Err(e) = fut.await {
            error!("{name} task failed: {e:?}");
        }
    })
}

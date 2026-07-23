#![feature(never_type)]

use tokio::task::JoinHandle;
use tracing::error;

pub mod bridge;
pub mod events;
pub mod setpoints;

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

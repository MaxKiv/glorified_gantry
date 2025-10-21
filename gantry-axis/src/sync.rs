use std::time::Duration;

use oze_canopen::interface::CanOpenInterface;
use tokio::{
    sync::broadcast,
    sync::watch,
    task::JoinHandle,
    time::{self, Instant},
};

use crate::spawn_logged;

pub struct SyncMasterHandle {
    handle: JoinHandle<()>,
    sync_rx: broadcast::Receiver<Instant>,
    sync_period_tx: watch::Sender<Duration>,
}

impl SyncMasterHandle {
    pub fn get_sync_receiver(&self) -> broadcast::Receiver<Instant> {
        self.sync_rx.resubscribe()
    }
}

pub struct SyncMaster {
    canopen: CanOpenInterface,
    period: Duration,
}

pub const DEFAULT_SYNC_PERIOD: Duration = Duration::from_millis(1000);

impl SyncMaster {
    pub fn init(canopen: CanOpenInterface) -> SyncMasterHandle {
        let (sync_tx, sync_rx) = tokio::sync::broadcast::channel(10);
        let (sync_period_tx, sync_period_rx) = watch::channel(DEFAULT_SYNC_PERIOD);

        let handle = spawn_logged("SYNC", async move {
            SyncMaster::sync_loop(sync_tx, sync_period_rx, canopen).await
        });

        SyncMasterHandle {
            handle,
            sync_rx,
            sync_period_tx,
        }
    }

    pub async fn sync_loop(
        sync_tx: broadcast::Sender<Instant>,
        mut sync_period_rx: watch::Receiver<Duration>,
        canopen: CanOpenInterface,
    ) -> anyhow::Result<()> {
        let Ok(_) = sync_period_rx.changed().await else {
            panic!(
                "Unable to establish SYNC loop in gantry - Unable to receive default SYNC period"
            );
        };

        let period = { sync_period_rx.borrow_and_update().clone() };

        let mut interval = time::interval(period);
        loop {
            interval.tick().await;

            // 1 send SYNC frame on bus
            canopen
                .send_sync()
                .await
                .map_err(|e| anyhow::anyhow!("SyncMaster unable to send SYNC: {e:?}"))?;

            // 2 broadcast to all drivers
            sync_tx.send(Instant::now()).map_err(|e| {
                anyhow::anyhow!(
                    "SyncMaster started new SYNC cycle but is unable to inform receivers: {e} "
                )
            })?;
        }
    }
}

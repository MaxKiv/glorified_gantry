use std::time::Duration;

use oze_canopen::interface::CanOpenInterface;
use tokio::{
    sync::{broadcast, watch},
    task::JoinHandle,
    time::{self, Instant, Interval},
};

use crate::spawn_logged;

pub struct SyncReceiver {
    pub sync_rx: broadcast::Receiver<Instant>,
}

pub struct SyncEnabler {
    pub sync_enable_tx: watch::Sender<bool>,
}

impl SyncReceiver {
    pub fn get_sync_receiver(&self) -> broadcast::Receiver<Instant> {
        self.sync_rx.resubscribe()
    }
}

pub struct SyncMaster {
    pub handle: JoinHandle<()>,
    canopen: CanOpenInterface,
    period: Duration,
}

pub const DEFAULT_SYNC_PERIOD: Duration = Duration::from_millis(100);

impl SyncMaster {
    pub fn init(
        canopen: CanOpenInterface,
        period: Duration,
    ) -> (SyncMaster, SyncEnabler, SyncReceiver) {
        let (sync_tx, sync_rx) = tokio::sync::broadcast::channel(10);
        let (sync_enable_tx, sync_enable_rx) = tokio::sync::watch::channel(false);

        let handle_canopen = canopen.clone();
        let handle = spawn_logged("SYNC", async move {
            SyncMaster::master_loop(sync_tx, sync_enable_rx, period, handle_canopen).await
        });

        let sync_master = SyncMaster {
            handle,
            canopen,
            period,
        };

        let sync_enabler = SyncEnabler { sync_enable_tx };

        let sync_receiver = SyncReceiver { sync_rx };

        (sync_master, sync_enabler, sync_receiver)
    }

    pub async fn master_loop(
        sync_tx: broadcast::Sender<Instant>,
        mut sync_enable: watch::Receiver<bool>,
        period: Duration,
        canopen: CanOpenInterface,
    ) -> anyhow::Result<()> {
        let mut interval = time::interval(period);

        tracing::info!("SyncMaster starting sync loop, waiting for sync_enable");
        loop {
            let enable = *sync_enable.borrow_and_update();
            if enable {
                interval.tick().await;

                SyncMaster::sync_loop_tick(&sync_tx, &canopen).await?;
            }
        }
    }

    pub async fn sync_loop_tick(
        sync_tx: &broadcast::Sender<Instant>,
        canopen: &CanOpenInterface,
    ) -> anyhow::Result<()> {
        // 1 send SYNC frame on bus
        canopen
            .send_sync()
            .await
            .map_err(|e| anyhow::anyhow!("SyncMaster unable to send SYNC: {e:?}"))?;

        // 2 broadcast SYNC send time to all drivers, indicating new SYNC was sent
        sync_tx.send(Instant::now()).map_err(|e| {
            anyhow::anyhow!(
                "SyncMaster started new SYNC cycle but is unable to inform receivers: {e} "
            )
        })?;

        Ok(())
    }
}

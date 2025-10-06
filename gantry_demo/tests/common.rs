use std::time::Duration;

use gantry_cia402::driver::{event::MotorEvent, nmt::NmtState};
use tokio::{
    sync::broadcast,
    time::{self, Instant},
};
use tracing::error;

pub async fn wait_for_event(
    mut event_rx: broadcast::Receiver<MotorEvent>,
    watch_for: MotorEvent,
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("timeout elapsed".to_owned());
        }

        let recv_future = event_rx.recv();
        let result = time::timeout(remaining, recv_future).await;

        match result {
            Ok(Ok(event)) => {
                if event == watch_for {
                    return Ok(());
                }
                // else keep looping for the next one
            }
            Ok(Err(broadcast::error::RecvError::Lagged(_))) => {
                // Messages were missed, continue to next one
                error!("Lagged in wait_for_event, indicates serious issue");
                return Err("Lagged".to_owned());
            }
            Ok(Err(broadcast::error::RecvError::Closed)) => {
                error!("Event channel closed in wait_for_event");
                return Err("event channel closed".to_owned());
            }
            Err(_) => {
                return Err("timeout elapsed".to_owned());
            }
        }
    }
}

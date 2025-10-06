pub mod comms;
pub mod driver;
pub mod error;
pub mod od;

use crate::driver::event::MotorEvent;
use oze_canopen::{canopen::RxMessage, interface::CanOpenInterface};
use tokio::sync::broadcast::{self, error::RecvError};
use tracing::*;

#[instrument(skip(event_rx))]
pub async fn log_events(
    mut event_rx: broadcast::Receiver<MotorEvent>,
    node_id: u8,
) -> Result<(), RecvError> {
    loop {
        tokio::select! {
            Ok(event) = event_rx.recv() => {
                info!("Received Feedback: {event:?}");
            },
            _ = tokio::signal::ctrl_c() => return Ok(()),
        };
    }
}

#[instrument(skip(canopen))]
pub async fn log_canopen(mut canopen: CanOpenInterface) -> Result<(), RecvError> {
    let can_name = canopen.connection.lock().await.can_name.clone();

    loop {
        tokio::select! {
            frame = canopen.rx.recv() => {
                match frame {
                    Ok(frame) => {
                        info!("{}\t{}", can_name, &format_frame(&frame));
                    }
                    Err(err) => {
                        error!("Error logging canopen traffic: {err}");
                    }
                }
            },
            _ = tokio::signal::ctrl_c() => return Ok(()),
        };
    }
}

fn format_frame(frame: &RxMessage) -> String {
    format!(
        "{}\t{}\t{:?}",
        frame.cob_id_to_string(),
        frame.dlc,
        format_data(&frame.data, frame.dlc)
    )
}

fn format_data(data: &[u8], dlc: usize) -> String {
    // let mut out = String::from("[");
    let mut out = String::new();
    for byte in &data[0..dlc] {
        out.push_str(&format!("{:#02x} ", byte));
    }
    // out += "]";

    out
}

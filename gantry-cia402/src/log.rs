use oze_canopen::{canopen::RxMessage, interface::CanOpenInterface};
use tokio::sync::broadcast::{self, error::RecvError};
use tracing::{instrument, *};

use crate::driver::{
    event::MotorEvent,
    receiver::frame::{Frame, MessageType, ParseError},
};

#[instrument(skip(event_rx))]
pub async fn log_events(
    mut event_rx: broadcast::Receiver<MotorEvent>,
    node_id: u8,
) -> Result<(), RecvError> {
    loop {
        tokio::select! {
            Ok(event) = event_rx.recv() => {
                info!(
                    target: "events",
                    data = %format!("{:?}", event)
                );
            },
            _ = tokio::signal::ctrl_c() => return Ok(()),
        };
    }
}

#[instrument(skip(canopen))]
pub async fn log_canopen_pretty(mut canopen: CanOpenInterface) -> Result<(), RecvError> {
    loop {
        tokio::select! {
            message = canopen.rx.recv() => {
                let span = span!(Level::TRACE, "sniffer");
                let _enter = span.enter();

                match message {
                    Ok(message) => {
                        let Ok(parsed): Result<Frame, _> = message.try_into() else {
                            error!("Error parsing message: {message:?}");
                            continue;
                        };
                        parsed.log();
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

#[instrument(skip(canopen))]
pub async fn log_canopen_raw(mut canopen: CanOpenInterface) -> Result<(), RecvError> {
    loop {
        tokio::select! {
            message = canopen.rx.recv() => {
                match message {
                    Ok(message) => {
                        info!("{}", &format_frame(&message));
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

pub fn format_frame(message: &RxMessage) -> String {
    format!(
        "can0\t{}\t{}\t{:?}",
        message.cob_id_to_string(),
        message.dlc,
        format_data(&message.data, message.dlc)
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

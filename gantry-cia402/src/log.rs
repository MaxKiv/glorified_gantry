use oze_canopen::{canopen::RxMessage, interface::CanOpenInterface};
use tokio::sync::broadcast::{self, error::RecvError};
use tracing::{instrument, *};

use crate::driver::{
    event::MotorEvent,
    feedback::frame::{Frame, ParseError},
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
            frame = canopen.rx.recv() => {
                let span = span!(Level::TRACE, "sniffer");
                let _enter = span.enter();

                match frame {
                    Ok(frame) => {
                        let parsed: Result<Frame, ParseError> = frame.try_into();
                        if let Ok(frame) = parsed {
                            frame.log();
                        } else {
                            error!("Error parsing frame: {frame:?}");
                        }
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
            frame = canopen.rx.recv() => {
                match frame {
                    Ok(frame) => {
                        info!("{}", &format_frame(&frame));
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

pub fn format_frame(frame: &RxMessage) -> String {
    format!(
        "can0\t{}\t{}\t{:?}",
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

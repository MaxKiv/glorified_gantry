use gantry_cia402::driver::{event::MotorEvent, feedback::frame::Frame};
use oze_canopen::{canopen::RxMessage, interface::CanOpenInterface};
use tokio::sync::broadcast::{self, error::RecvError};
use tracing::{Level, *};
use tracing_subscriber::FmtSubscriber;

use std::fmt::Debug;
use std::time::Duration;

use tokio::time::{self, Instant};
use tracing_subscriber::{
    fmt::{self, format::Writer},
    registry::LookupSpan,
};

use owo_colors::OwoColorize;
use tracing::field::{Field, Visit};
use tracing_subscriber::fmt::*;

pub fn setup_tracing() {
    // Setup tracing
    let subscriber = FmtSubscriber::builder()
        .event_format(FrameFormatter) // use our formatter
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed");
}

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
pub async fn log_canopen_pretty(mut canopen: CanOpenInterface) -> Result<(), RecvError> {
    loop {
        tokio::select! {
            frame = canopen.rx.recv() => {
                match frame {
                    Ok(frame) => {
                        let parsed: Result<Frame, ()> = frame.try_into();
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

fn format_frame(frame: &RxMessage) -> String {
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

// a tiny visitor to extract a few fields from the Event

#[derive(Default)]
struct FieldExtractor {
    frame: Option<String>,
    node: Option<u64>,
    data: Option<String>,
    message: Option<String>,
}

impl Visit for FieldExtractor {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "frame" => self.frame = Some(value.to_string()),
            "message" => self.message = Some(value.to_string()),
            "data" => self.data = Some(value.to_string()),
            _ => {}
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "node" {
            self.node = Some(value);
        }
        if field.name() == "num" {
            // optionally stash num in data or similar
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        // fallback for non-str fields
        if field.name() == "frame" {
            self.frame = Some(format!("{:?}", value));
        }
        if field.name() == "message" && self.message.is_none() {
            self.message = Some(format!("{:?}", value));
        }
        if field.name() == "data" && self.data.is_none() {
            self.data = Some(format!("{:?}", value));
        }
    }
}

// custom formatter that uses writer.has_ansi_escapes()
struct FrameFormatter;

impl<S, N> FormatEvent<S, N> for FrameFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> core::fmt::Result {
        // extract some fields
        let mut ex = FieldExtractor::default();
        event.record(&mut ex);

        let supports_color = writer.has_ansi_escapes();

        let frame = ex.frame.unwrap_or_else(|| "UNKNOWN".to_string());
        let node = ex.node.unwrap_or(0);
        let message = ex.message.unwrap_or_default();
        let data = ex.data.unwrap_or_default();

        if supports_color {
            match frame.as_str() {
                "EMCY" => write!(
                    writer,
                    "{} from {}  {}",
                    "EMCY".red().bold(),
                    format!("Node {}", node).red(),
                    message
                )?,
                "TPDO" => write!(
                    writer,
                    "{} from {}  [{}]",
                    "TPDO".green().bold(),
                    format!("Node {}", node).green(),
                    data
                )?,
                "RPDO" => write!(
                    writer,
                    "{} to {}  [{}]",
                    "RPDO".purple().bold(),
                    format!("Node {}", node).purple(),
                    data
                )?,
                "SYNC" => write!(writer, "{}", "SYNC".white().bold())?,
                "NmtControl" => write!(
                    writer,
                    "{} for  {} request: {}",
                    "NMT-Control".bright_yellow().bold().reversed(),
                    format!("Node {}", node).bright_yellow(),
                    data.bright_yellow().reversed()
                )?,
                "NmtMonitor" => write!(
                    writer,
                    "{} from {} reports: {}",
                    "NMT-Monitor".yellow().bold(),
                    format!("Node {}", node).yellow(),
                    data.yellow().bold()
                )?,
                _ => write!(writer, "{} {}", frame, message)?,
            }
        } else {
            // plain fallback for files / non-ttys
            match frame.as_str() {
                "EMCY" => write!(writer, "EMCY from Node {}  {}", node, message)?,
                "TPDO" => write!(writer, "TPDO from Node {}  [{}]", node, data)?,
                _ => write!(writer, "{} {}", frame, message)?,
            }
        }

        writeln!(writer)
    }
}

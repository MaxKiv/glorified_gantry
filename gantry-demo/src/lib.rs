use gantry_cia402::driver::{event::MotorEvent, receiver::frame::MessageType};
use oze_canopen::{canopen::RxMessage, interface::CanOpenInterface};
use tokio::sync::broadcast::{self, error::RecvError};
use tracing::{Level, *};
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::fmt::time::SystemTime;
use tracing_subscriber::{
    FmtSubscriber, Registry, filter, fmt::time::Uptime, layer::SubscriberExt,
};
use tracing_subscriber::{Layer, filter::LevelFilter, prelude::*};

use std::fmt::Debug;
use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use owo_colors::OwoColorize;
use tokio::time::{self, Instant};
use tracing::field::{Field, Visit};
use tracing_subscriber::fmt::*;
use tracing_subscriber::{
    fmt::{self, format::Writer},
    registry::LookupSpan,
};

pub fn setup_tracing() {
    // your custom formatter for canopen

    let frame_fmt_layer = tracing_subscriber::fmt::layer()
        .event_format(FrameFormatter)
        .with_filter(filter_fn(|meta| meta.target() == "canopen"));

    let default_layer =
        tracing_subscriber::fmt::layer().with_filter(filter_fn(|meta| meta.target() != "canopen"));

    let subscriber = Registry::default()
        .with(default_layer)
        .with(frame_fmt_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed");
}

// a tiny visitor to extract a few fields from the Event
#[derive(Default)]
struct FieldExtractor {
    frame: Option<String>,
    node: Option<u64>,
    data: Option<String>,
    message: Option<String>,
    parsed: Option<String>,
    index: Option<String>,
    sub_index: Option<String>,
    num: Option<u64>,
}

impl Visit for FieldExtractor {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "frame" => self.frame = Some(value.to_string()),
            "message" => self.message = Some(value.to_string()),
            "data" => self.data = Some(value.to_string()),
            "parsed" => self.parsed = Some(value.to_string()),
            "index" => self.index = Some(value.to_string()),
            "sub_index" => self.sub_index = Some(value.to_string()),
            _ => {}
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "node" {
            self.node = Some(value);
        }
        if field.name() == "num" {
            self.num = Some(value)
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
        if field.name() == "parsed" && self.data.is_none() {
            self.parsed = Some(format!("{:?}", value));
        }
        if field.name() == "index" && self.data.is_none() {
            self.index = Some(format!("{:?}", value));
        }
        if field.name() == "sub_index" && self.data.is_none() {
            self.sub_index = Some(format!("{:?}", value));
        }
    }
}

// Custom formatter for [`Frame`]
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
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true);

        // print the timestamp first
        write!(writer, "{} ", timestamp.dimmed())?;

        write!(writer, "{} ", "SNIFF".white().bold())?;

        // extract some fields using our FieldExtractor
        let mut ex = FieldExtractor::default();
        event.record(&mut ex);

        let supports_color = writer.has_ansi_escapes();

        let frame = ex.frame.unwrap_or_else(|| "UNKNOWN".to_string());
        let node = ex.node.unwrap_or(0);
        let num = ex.num.unwrap_or(0);
        let message = ex.message.unwrap_or_default();
        let data = ex.data.unwrap_or_default();
        let parsed = ex.parsed.unwrap_or_default();
        let index = ex.index.unwrap_or_default();
        let sub_index = ex.sub_index.unwrap_or_default();

        if supports_color {
            match frame.as_str() {
                "EMCY" => write!(
                    writer,
                    "{} from {} {}",
                    "EMCY".red().bold(),
                    format!("Node {}", node).red(),
                    message
                )?,
                "TPDO" => write!(
                    writer,
                    "{} {} <- {} [{}]",
                    "TPDO".green(),
                    num.green().bold(),
                    format!("Node {}", node).green(),
                    data
                )?,
                "RPDO" => write!(
                    writer,
                    "{} {} -> {} [{}]",
                    "RPDO".purple(),
                    num.purple().bold(),
                    format!("Node {}", node).purple(),
                    data
                )?,
                "RSDO" => write!(
                    writer,
                    "{} -> {} [{}] => {}",
                    "RSDO".bright_blue().bold(),
                    format!("Node {}", node).bright_blue(),
                    data,
                    parsed.blue(),
                )?,
                "TSDO" => write!(
                    writer,
                    "{} <- {} {}",
                    "TSDO".cyan().bold(),
                    format!("Node {}", node).cyan(),
                    parsed.cyan(),
                )?,
                "SYNC" => write!(writer, "{}", "SYNC".white().bold())?,
                "NmtControl" => write!(
                    writer,
                    "{} -> {} request {}",
                    "NMT-Control".bright_yellow().bold().reversed(),
                    format!("Node {}", node).bright_yellow(),
                    data.bright_yellow().reversed()
                )?,
                "NmtMonitor" => write!(
                    writer,
                    "{} <- {} reports {}",
                    "NMT-Monitor".yellow().bold(),
                    format!("Node {}", node).yellow(),
                    data.yellow().bold()
                )?,
                _ => write!(writer, "{} {}", frame, message)?,
            }
        } else {
            write!(writer, "Writer does not support colors :<")?;
        }

        writeln!(writer)
    }
}

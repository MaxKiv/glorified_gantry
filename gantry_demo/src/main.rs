use gantry_cia402::{
    comms::{pdo::mapping::PdoMapping, sdo::SdoAction},
    driver::{Cia402Driver, event::MotorEvent},
    od::ObjectDictionary,
};
use tokio::sync::broadcast::{self, error::RecvError};
use tracing::{Level, *};
use tracing_subscriber::FmtSubscriber;

const NODE_ID: u8 = 3;

const PARAMS: [SdoAction; 1] = [SdoAction::Upload {
    index: ObjectDictionary::DEVICE_TYPE.index,
    subindex: ObjectDictionary::DEVICE_TYPE.sub_index,
}];

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    // Setup tracing
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed");

    info!("Starting can interface");
    let (canopen, handles) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

    info!("Initializing cia402 driver");
    let drive = Cia402Driver::init(
        NODE_ID,
        canopen,
        &PARAMS,
        PdoMapping::CUSTOM_RPDOS,
        PdoMapping::CUSTOM_TPDOS,
    )
    .await
    .expect("unable to construct Cia402 driver");

    if let Err(err) = log_events(drive.event_rx).await {
        error!("Error logging events from binary crate: {err}");
    }
}

async fn log_events(mut event_rx: broadcast::Receiver<MotorEvent>) -> Result<(), RecvError> {
    loop {
        let event = event_rx.recv().await?;
        info!("Received Feedback: {event:?}")
    }
}

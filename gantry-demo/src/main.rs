use ::tracing::info;
use gantry_cia402::{comms::sdo::SdoAction, log::log_canopen_pretty, od::DEVICE_TYPE};
use gantry_demo::setup_tracing;
use oze_canopen::canopen;
use tracing::*;

const NODE_ID: u8 = 3;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    setup_tracing();

    info!("Starting can interface");
    let (canopen, handles) = canopen::start(String::from("can0"), Some(1000000));

    let _ = log_canopen_pretty(canopen).await;
}

use ::tracing::info;
use gantry_cia402::{comms::sdo::SdoAction, log::log_canopen_pretty, od::DEVICE_TYPE};
use gantry_demo::setup_tracing;
use tracing::*;

const NODE_ID: u8 = 3;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    setup_tracing();

    info!("Starting can interface");
    let (canopen, handles) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

    // info!("Initializing cia402 driver");
    // let drive = Cia402Driver::init(
    //     NODE_ID,
    //     canopen,
    //     &PARAMS,
    //     PdoMapping::CUSTOM_RPDOS,
    //     PdoMapping::CUSTOM_TPDOS,
    // )
    // .await
    // .expect("unable to construct Cia402 driver");

    let _ = log_canopen_pretty(canopen).await;
}

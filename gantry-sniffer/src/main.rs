use ::tracing::info;
use gantry_cia402::log::log_canopen_pretty;
use gantry_sniffer::setup_tracing;
use oze_canopen::canopen;
use tokio::sync::broadcast::error::RecvError;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<(), RecvError> {
    setup_tracing();

    info!("Starting can interface");
    let (canopen, _handles) = canopen::start(String::from("can0"), Some(1_000_000));

    info!("Starting to sniff");
    log_canopen_pretty(canopen).await?;

    Ok(())
}

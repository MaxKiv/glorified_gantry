use std::time::Duration;

use oze_canopen::{
    error::CoError,
    interface::CanOpenInterface,
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
};
use tracing::{Level, *};
use tracing_subscriber::FmtSubscriber;

const NODE_ID: u8 = 3;

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
    let (interface, mut handles) = oze_canopen::canopen::start(String::from("can0"), Some(1000000));

    // Run quick test
    if let Err(err) = run(interface).await {
        error!("Test failed: {err:?}");
    } else {
        info!("Test success!");
    }

    // stop tasks
    handles.close_and_join().await;
}

async fn run(interface: CanOpenInterface) -> Result<(), CoError> {
    info!("Starting test, Sending NMT Operational to node id {NODE_ID}");
    // Motor boots into NMT::PreOperational -> Set motor to NMT::Operational
    interface
        .send_nmt(NmtCommand::new(
            NmtCommandSpecifier::StartRemoteNode,
            NODE_ID,
        ))
        .await?;

    // Give the slave device some time to boot, we all have trouble getting out of bed sometimes
    tokio::time::sleep(Duration::from_millis(200)).await;

    info!("Getting sdo client");
    let s = interface.get_sdo_client(3).unwrap();

    info!("Testing upload");
    let dat = s.lock().await.upload(0x1000, 0).await?;
    info!("Test upload - device type: {dat:?}");

    info!("Testing download");
    let dat = s.lock().await.upload(0x607A, 0).await?;
    info!("Test before download - target position: {dat:?} - Setting to 0x0");

    s.lock()
        .await
        .download(0x607A, 0, &[0x00, 0x00, 0x00, 0x00])
        .await?;
    let dat = s.lock().await.upload(0x607A, 0).await?;
    info!("Test before download - target position: {dat:?} - Setting to 0x1");

    s.lock()
        .await
        .download(0x607A, 0, &[0x01, 0x01, 0x01, 0x01])
        .await?;
    let dat = s.lock().await.upload(0x607A, 0).await?;
    info!("Test after download - target position: {dat:?}");

    Ok(())
}

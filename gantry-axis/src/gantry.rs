use tracing::*;

use crate::sync::{SyncMaster, SyncMasterHandle};

pub struct Gantry {
    sync: SyncMasterHandle,
}

impl Gantry {
    fn new() -> Self {
        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        info!("Starting SYNC Master");
        let sync = SyncMaster::init(canopen);

        Self { sync }
    }
}

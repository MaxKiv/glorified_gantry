use std::sync::Arc;

use tokio::sync::{Mutex, mpsc::Sender};
use tracing::*;

use crate::od::state::Cia402State;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct StatusWord: u16 {
        /// Bit 0: Ready to switch on
        const READY_TO_SWITCH_ON   = 1 << 0;
        /// Bit 1: Switched on
        const SWITCHED_ON          = 1 << 1;
        /// Bit 2: Operation enabled
        const OPERATION_ENABLED    = 1 << 2;
        /// Bit 3: Fault
        const FAULT                = 1 << 3;
        /// Bit 4: Voltage enabled
        const VOLTAGE_ENABLED      = 1 << 4;
        /// Bit 5: Quick stop
        const QUICK_STOP           = 1 << 5;
        /// Bit 6: Switch on disabled
        const SWITCH_ON_DISABLED   = 1 << 6;
        /// Bit 7: Warning
        const WARNING              = 1 << 7;

        /// Bit 8: Manufacturer specific
        const MANUFACTURER_1       = 1 << 8;
        /// Bit 9: Remote (drive is under control via fieldbus)
        const REMOTE               = 1 << 9;
        /// Bit 10: Target reached (depends on operation mode)
        const TARGET_REACHED       = 1 << 10;
        /// Bit 11: Internal limit active
        const INTERNAL_LIMIT       = 1 << 11;

        /// Bit 12: Manufacturer specific
        const MANUFACTURER_2       = 1 << 12;
        /// Bit 13: Manufacturer specific
        const MANUFACTURER_3       = 1 << 13;
        /// Bit 14: Manufacturer specific
        const MANUFACTURER_4       = 1 << 14;
        /// Bit 15: Manufacturer specific
        const MANUFACTURER_5       = 1 << 15;
    }
}
const DEFAULT_STATUS: StatusWord = StatusWord::empty();

/// Responsible for all CANopen communication to the drive
/// Receives updates from the Cia402 state machine and operational mode specific handler
/// It encodes these changes into the appropriate controlword bits or OD object
/// It then sends these changes out on the CANopen bus using the accessor
pub async fn receive_status<A: ObjectDictionaryAccessor + 'static>(
    mut accessor: Arc<Mutex<A>>,
    mut state_tx: Sender<Cia402State>,
) {
    let status;
    if let Ok(boot_status) = accessor.lock().await.read_statusword().await {
        status = match StatusWord::from_bits(boot_status) {
            Some(parsed) => parsed,
            None => DEFAULT_STATUS,
        }
    } else {
        error!(
            "Error reading statusword on boot, defaulting to {:#16b}",
            status.bits()
        );
    }

    loop {
        trace!("1. Read statusword");
        match accessor.lock().await.read_statusword().await {
            Ok(read_status) => {
                trace!("2. Parse statusword");
                match StatusWord::from_bits(read_status) {
                    Some(parsed_status) => {
                        if parsed_status != status {
                            trace!(
                                "3. New status received: {parsed_status:?}, notifying listeners"
                            );
                            if let Err(err) = state_tx.send(read_status).await {
                                error!("Error sending statusword update: {err}");
                            }
                            read_status = status;
                        }
                    }
                    None => error!(
                        "Unable to parse received status bits {:#16b} into StatusWord bitflags",
                        read_status
                    ),
                }
            }
            Err(err) => error!("Unable to read statusword: {err}"),
        }
    }
}

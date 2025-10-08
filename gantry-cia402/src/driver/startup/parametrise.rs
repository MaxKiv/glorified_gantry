use std::{sync::Arc, time::Duration};

use anyhow::Result;
use oze_canopen::sdo_client::SdoClient;
use tokio::sync::Mutex;
use tracing::*;

use crate::comms::sdo::SdoAction;

const SDO_PROCESS_DURATION: Duration = Duration::from_millis(0); // Typical SDO round trip at 1mbit/s ~= 4ms, + engineering factor :)

/// Parametrize the motor at given node id
/// parametrisation is the process of setting important parameters like
/// maximum velocity or torque to known values at boot
/// The motor usually does not commit these changes to non-volatile memory,
/// so this has to run on every new boot cycle of the device
pub async fn parametrise_motor(
    node_id: u8,
    parameters: &[SdoAction<'_>],
    sdo: Arc<Mutex<SdoClient>>,
) -> Result<()> {
    trace!("Starting parametrisation of Motor with node id {}", node_id);

    // parametrisation is done through a series of SDO calls, perform these in order
    for action in parameters {
        trace!("parametrizing node id {} with: {action:?}", node_id);
        if let Err(err) = action.run_on_sdo_client(sdo.clone()).await {
            error!("Error while parametrizing node id {}: {err}", node_id);
        }

        tokio::time::sleep(SDO_PROCESS_DURATION).await;
    }

    Ok(())
}

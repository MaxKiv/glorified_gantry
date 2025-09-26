use anyhow::Result;
use tokio::task;

use crate::comms::pdo::mapping::PdoMapping;

pub async fn manage_pdo(pdo_mappings: &'_ [&'_ PdoMapping<'_>]) -> Result<()> {
    configure_pdo_mapping(pdo_mappings);

    task::spawn(manage_tpdo());

    Ok(())
}

/// Configures the CiA402 PDO mapping
/// Continously tracks the PDO-mapped state, for example fetches the statusword info
/// Manages PDO-mapped output, like the controlword bits to manipulate the cia402 state machine
pub async fn manage_tpdo() -> Result<()> {
    loop {
        // await changes that need to be reflected in TPDO
    }
}

async fn configure_pdo_mapping(pdo_mappings: _) -> _ {
    todo!()
}

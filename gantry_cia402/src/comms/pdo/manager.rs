use anyhow::Result;
use tokio::task;

pub async fn manage_pdo(pdo_mappings: _) -> Result<()> {
    configure_pdo_mapping(pdo_mappings);

    task::spawn(manage_tpdo());
    task::spawn(manage_rpdo());
}

/// Configures the CiA402 PDO mapping
/// Continously tracks the PDO-mapped state, for example fetches the statusword info
/// Manages PDO-mapped output, like the controlword bits to manipulate the cia402 state machine
pub async fn manage_tpdo() -> Result<()> {
    loop {
        // await changes that need to be reflected in TPDO
    }
}

pub async fn manage_rpdo() -> Result<()> {
    loop {
        // await changes that need to be reflected in TPDO
        tokio::select! {
            _ = ctrl_c() => return,
            _ = sleep(Duration::from_millis(100)) => {},
        };
    }
}

async fn configure_pdo_mapping(pdo_mappings: _) -> _ {
    todo!()
}

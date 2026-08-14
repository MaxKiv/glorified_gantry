use crate::{
    comms::pdo::mapping::{
        PdoSet,
        cyclic_synchronous::{CSP_PDOS, CST_PDOS, CSV_PDOS},
        default::DEFAULT_PDOS,
    },
    driver::oms::OperationMode,
};

#[derive(Debug)]
pub struct PdoTable {
    table: [Option<&'static PdoSet>; std::mem::variant_count::<OperationMode>()],
}

impl PdoTable {
    pub fn get_default_pdoset(&self) -> &'static PdoSet {
        self.table[OperationMode::ProfilePosition.as_index()].unwrap_or(&DEFAULT_PDOS)
    }

    pub fn get_pdoset_for_operationmode(&self, mode: OperationMode) -> &'static PdoSet {
        self.table[mode.as_index()].unwrap_or(&DEFAULT_PDOS)
    }
}

/// Required [`PDOSet`] mapping lookup table for each [`OperationMode`]
/// NOTE:  None effectively disables the usage of the mode
///        All modes set to use [`DEFAULT_PDOS`] can be used without reconfiguring PDO mapping & dropping to NMT Pre-OP
pub const DEFAULT_PDO_TABLE: PdoTable = PdoTable {
    table: [
        None,                // Auto setup
        None,                // Clock direction
        None,                // No Change
        Some(&DEFAULT_PDOS), // Profile Position
        Some(&DEFAULT_PDOS), // Profile Velocity
        Some(&DEFAULT_PDOS), // Velocity
        Some(&DEFAULT_PDOS), // Profile Torque
        None,                // Reserved
        Some(&DEFAULT_PDOS), // Homing
        None,                // Interpolated Position
        Some(&CSP_PDOS),     // Cyclic Synchronous Position
        Some(&CSV_PDOS),     // Cyclic Synchronous Velocity
        Some(&CST_PDOS),     // Cyclic Synchronous Torque
    ],
};

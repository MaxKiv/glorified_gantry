use tracing::*;

use crate::{
    canopen::{
        nmt::{
            NmtCommandSpecifier::{self, EnterPreOperational, StartRemoteNode},
            NmtState::{self, Operational, PreOperational},
        },
        od::{
            RPDO_COMMUNICATION_PARAMETER_BASE_INDEX, RPDO_MAPPING_PARAMETER_BASE_INDEX,
            TPDO_COMMUNICATION_PARAMETER_BASE_INDEX, TPDO_MAPPING_PARAMETER_BASE_INDEX,
            calculate_pdo_index_offset,
        },
        pdo::{
            PdoType, TransmissionType,
            mapping::{OMSNodePdoConfig, PdoMapping},
        },
    },
    cia402::Cia402Identifier,
    motor::script::{
        Script,
        ScriptStep::{self, *},
    },
};

// PDO mapping validity toggle
const VALID_BIT: u32 = 1 << 31;

// PDO communication parameter sub-indices (CiA 301)
const COMM_SUB_COB_ID: u8 = 0x01;
const COMM_SUB_TX_TYPE: u8 = 0x02;
const COMM_SUB_INHIBIT: u8 = 0x03;
const COMM_SUB_EVENT_TIMER: u8 = 0x05;

// See datasheet p.122 & p.202. Tuning knobs for TPDO bus load.
const EVENT_TIMER_MS: u16 = 1;
const INHIBIT_TIME_100US: u16 = 500; // 500 * 100us = 50 ms

/// CiA 301 predefined PDO COB-ID
/// NOTE: only valid for PDO 0..=3 (PDO1..PDO4).
fn default_cob_id(pdo: &PdoType, num: u8, node_id: u8) -> u32 {
    debug_assert!(num < 4, "no predefined COB-ID for PDO {num}");
    let base = match pdo {
        PdoType::RPDO => 0x200,
        PdoType::TPDO => 0x180,
    };
    base + 0x100 * num as u32 + node_id as u32
}

fn pdo_invalidate_step(
    id: &Cia402Identifier,
    node_id: u8,
    pdo_type: PdoType,
    num: u8,
) -> ScriptStep {
    let (comm_idx, _) = get_pdo_indices(&pdo_type, num);
    let cob_id = default_cob_id(&pdo_type, num, node_id);
    ScriptStep::sdo_write(
        id.node_id,
        comm_idx,
        COMM_SUB_COB_ID,
        &(cob_id | VALID_BIT).to_le_bytes(),
    )
}
/// Get a [Vec<ScriptStep>] containing all actions required to switch a [CiA402Motor] to a new [OMSNodePdoConfig]
pub fn get_pdo_remap_script(
    id: &Cia402Identifier,
    old: &OMSNodePdoConfig,
    new: &OMSNodePdoConfig,
) -> Script {
    /// Steps to take before PDO mapping change
    pub const PDO_REMAP_START: [ScriptStep; 2] = [
        QuickStop,
        Nmt {
            cmd: EnterPreOperational,
            expect: PreOperational,
        },
    ];

    /// Steps to take after PDO mapping change
    pub const PDO_REMAP_END: [ScriptStep; 2] = [
        Nmt {
            cmd: StartRemoteNode,
            expect: Operational,
        },
        EnableOperation,
    ];

    let mut steps = Vec::with_capacity(64);

    steps.extend(PDO_REMAP_START);

    steps.extend(get_pdo_cfg_set_remapping_sdo_steps(id, old, new));

    steps.extend(PDO_REMAP_END);

    Script::new(steps)
}

/// Get a [Vec<ScriptStep>] containing all SDO writes required to switch a [CiA402Motor] to a new [OMSNodePdoConfig]
fn get_pdo_cfg_set_remapping_sdo_steps(
    node: &Cia402Identifier,
    old_pdo_cfg: &OMSNodePdoConfig,
    new_pdo_cfg: &OMSNodePdoConfig,
) -> Vec<ScriptStep> {
    let node_id = node.node_id.u8();
    let mut steps = Vec::with_capacity(64 * 8);

    // Check new RPDO mappings
    for (num, slot) in new_pdo_cfg.rpdo.iter().enumerate() {
        match slot {
            Some(mapping) => {
                steps.extend(get_single_pdo_remapping_script(node, num as u8, mapping))
            }
            None => {
                // Mapped before but not any more -> must be switched off,
                // otherwise the drive keeps producing/expecting it.
                if old_pdo_cfg.rpdo[num].is_some() {
                    steps.push(pdo_invalidate_step(node, node_id, PdoType::RPDO, num as u8));
                }
            }
        }
    }
    for (num, slot) in new_pdo_cfg.tpdo.iter().enumerate() {
        match slot {
            Some(mapping) => {
                steps.extend(get_single_pdo_remapping_script(node, num as u8, mapping))
            }
            None => {
                if old_pdo_cfg.tpdo[num].is_some() {
                    steps.push(pdo_invalidate_step(node, node_id, PdoType::TPDO, num as u8));
                }
            }
        }
    }

    steps
}

fn get_single_pdo_remapping_script(
    node: &Cia402Identifier,
    num: u8,
    new_pdo_cfg: &PdoMapping,
) -> Vec<ScriptStep> {
    let id = node.node_id;
    let pdo_type = new_pdo_cfg.pdo.clone();
    let cob_id = default_cob_id(&pdo_type, num, id.u8());
    let (communication_index, mapping_index) = get_pdo_indices(&pdo_type, num);
    let mut s = Vec::with_capacity(8 + new_pdo_cfg.sources.len());

    info!(
        "Building Pdo remap script for node {} - {:?} #{num}: {:?}",
        id.u8(),
        pdo_type,
        new_pdo_cfg
    );

    trace!(
        "1.A Deactivate the PDO by setting the Valid Bit (bit 31) of subindex
        01h of the corresponding PDO communication parameter ({}) to \"1\".",
        communication_index,
    );
    s.push(ScriptStep::sdo_write(
        id,
        communication_index,
        COMM_SUB_COB_ID,
        &(cob_id | VALID_BIT).to_le_bytes(),
    ));

    trace!(
        "1.B Set Transmission type to {:?}",
        new_pdo_cfg.transmission_type
    );
    s.push(ScriptStep::sdo_write(
        id,
        communication_index,
        COMM_SUB_TX_TYPE,
        &[new_pdo_cfg.transmission_type.od_value()],
    ));

    // 1.C & 1.D Configure periodic event + inhibit time if TransmissionType = on change
    if new_pdo_cfg.transmission_type == TransmissionType::OnChange {
        trace!("1.C / 1.D TPDO on-change: periodic event + inhibit time to bound bus load");
        s.push(ScriptStep::sdo_write(
            id,
            communication_index,
            COMM_SUB_EVENT_TIMER,
            &EVENT_TIMER_MS.to_le_bytes(),
        ));
        s.push(ScriptStep::sdo_write(
            id,
            communication_index,
            COMM_SUB_INHIBIT,
            &INHIBIT_TIME_100US.to_le_bytes(),
        ));
    }

    // 2. Deactivate the mapping by setting subindex 00h of the corresponding mapping parameter to \"0\".,
    trace!(
        "2. Deactivate the mapping by setting subindex 00h of the corresponding mapping parameter ({}) to \"0\".",
        mapping_index
    );
    s.push(ScriptStep::sdo_write(id, mapping_index, 0x00, &[0]));

    trace!("3. Change the mapping in the desired subindices.");
    for (i, source) in new_pdo_cfg.sources.iter().enumerate() {
        let entry: u32 = ((source.entry.index as u32) << 16)
            | ((source.entry.sub_index as u32) << 8)
            | (source.len as u32);
        s.push(ScriptStep::sdo_write(
            id,
            mapping_index,
            (i + 1) as u8,
            &entry.to_le_bytes(),
        ));
    }

    trace!(
        "4. Activate the mapping by writing the number of objects that are to be mapped in subindex 00h of the corresponding mapping parameter (e.g., 1600h:00h)."
    );
    s.push(ScriptStep::sdo_write(
        id,
        mapping_index,
        0x0,
        &[new_pdo_cfg.sources.len() as u8],
    ));

    trace!(
        "5. Activate the PDO by setting bit 31 of subindex 01h of the corresponding communication parameter (e.g., 1400h:01h) to \"0\"."
    );
    s.push(ScriptStep::sdo_write(
        id,
        communication_index,
        COMM_SUB_COB_ID,
        &cob_id.to_le_bytes(),
    ));

    s
}

fn get_pdo_indices(pdo: &PdoType, num: u8) -> (u16, u16) {
    match pdo {
        PdoType::RPDO => (
            calculate_pdo_index_offset(RPDO_COMMUNICATION_PARAMETER_BASE_INDEX, num),
            calculate_pdo_index_offset(RPDO_MAPPING_PARAMETER_BASE_INDEX, num),
        ),
        PdoType::TPDO => (
            calculate_pdo_index_offset(TPDO_COMMUNICATION_PARAMETER_BASE_INDEX, num),
            calculate_pdo_index_offset(TPDO_MAPPING_PARAMETER_BASE_INDEX, num),
        ),
    }
}

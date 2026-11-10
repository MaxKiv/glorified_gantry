pub mod error;
pub mod limits;
pub mod pdo;
pub mod script;
pub mod state;

use std::time::Instant;

use crate::canopen::nmt::NmtCommandSpecifier::EnterPreOperational;
use crate::canopen::nmt::NmtCommandSpecifier::StartRemoteNode;
use crate::canopen::nmt::NmtState::Operational;
use crate::canopen::od::TPDO_MAPPING_PARAMETER_BASE_INDEX;
use crate::canopen::od::entry::PdoSemantic;
use crate::canopen::od::value::ODValue;
use crate::canopen::pdo::RawRpdo;
use crate::canopen::pdo::mapping::PdoValue;
use crate::cw::ControlWord;
use crate::motor::limits::Cia402MotorLimits;
use crate::motor::script::Purpose;
use crate::motor::script::Script;
use crate::motor::script::ScriptStep;
use crate::motor::script::opmode_switch::get_pdo_remap_script;
use crate::rt::MotorFeedback;
use crate::sw::StatusWord;
use tracing::debug;
use tracing::{error, info, trace, warn};
use uom::si::length::millimeter;
use uom::si::torque::newton_meter;
use uom::si::velocity::meter_per_second;

/// Collection of up to 4 motor specific RPDO
pub type MotorRpdoSet = [Option<RawRpdo>; 4];

use crate::{
    canopen::{
        CanOpen, CanOpenError, MessageType,
        frame::CanOpenFrame,
        nmt::{
            NmtCommandSpecifier, NmtMonitorMessage,
            NmtState::{self, PreOperational},
        },
        od::{
            RPDO_COMMUNICATION_PARAMETER_BASE_INDEX, TPDO_COMMUNICATION_PARAMETER_BASE_INDEX,
            get_pdo_deactivation_od_entry,
        },
        pdo::{
            PdoType,
            mapping::{OMSNodePdoConfig, PdoMapping},
        },
        sdo::manager::{SdoCommand, SdoManager},
    },
    cia402::{Cia402Identifier, Cia402State},
    consts::pdo::NodePdoConfig,
    motor::{error::MotorError, state::MotorState},
    oms::{OperationMode, setpoint::Setpoint},
};
/// State of the Cia402Motor Controller
enum ControllerState {
    Idle,
    Operating,
    Configuring(Script, Purpose), // ie. Parametrising or Mode Switching
    Faulted(MotorError),
}

pub struct Cia402Motor {
    pub id: Cia402Identifier,

    // TODO: make this a reference
    canopen: CanOpen,

    sdo: SdoManager,

    controller_state: ControllerState,
    tpdo_rx: [bool; 4],

    cia402_state: Cia402State,             // Current CiA402 state
    nmt: NmtState,                         // Current NMT State
    opmode: OperationMode,                 // Active operation mode
    active_cfg: &'static OMSNodePdoConfig, // Current Pdo Mapping
    pdo_cfg: &'static NodePdoConfig,       // Required Pdo mapping per node

    // TODO, these 2 seem similar?
    pub motor_state: MotorState,
    feedback: MotorFeedback,

    active_setpoint: Setpoint, // Active motor setpoint
    controlword: ControlWord,  // Active controlword
    statusword: StatusWord,    // Last received statusword

    // Collection of latest setpoint per relevant mode
    target_position: i32,
    target_velocity: i32,
    target_torque: i16,

    limits: Cia402MotorLimits,                 // Profile limits
    default_parameters: &'static [SdoCommand], // All configuration parameters
}

pub struct TickCtx<'a> {
    now: Instant,
    io: &'a mut CanOpen,
    sdo_slot_free: bool,
}

impl Cia402Motor {
    pub fn new(
        id: Cia402Identifier,
        canopen: CanOpen,
        pdo_cfg: &'static NodePdoConfig,
        default_parameters: &'static [SdoCommand],
    ) -> Self {
        let nmt = NmtState::Bootup;
        let opmode = OperationMode::default();
        let active_setpoint = Setpoint::default();
        let cia402_state = Cia402State::default();
        let active_cfg = pdo_cfg.get_cfg_for_opmode(&opmode);

        Self {
            id,
            nmt,
            canopen,
            opmode,
            active_setpoint,
            cia402_state,
            pdo_cfg,
            active_cfg,
            controller_state: ControllerState::Idle,
            default_parameters,
            motor_state: MotorState::new(),
            feedback: MotorFeedback::new(),
            tpdo_rx: [false; 4],
            sdo: todo!(),
            target_position: todo!(),
            target_velocity: todo!(),
            target_torque: todo!(),
            limits: todo!(),
            controlword: todo!(),
            statusword: todo!(),
        }
    }

    pub fn request_nmt_command(&mut self, cmd: NmtCommandSpecifier) -> Result<(), CanOpenError> {
        self.canopen.send_nmt(cmd, &self.id)?;

        Ok(())
    }

    pub fn process_nmt_message(&mut self, msg: NmtMonitorMessage) {
        assert!(msg.node_id == self.id.node_id);
        self.nmt = msg.current_state;
    }

    /// Switches drive [`OperationMode`], remapping pdo if required
    /// NOTE: this must be called at the start of a cycle
    pub fn switch_operation_mode(&mut self, new: &OperationMode) -> Result<(), MotorError> {
        // Is a pdo remapping required?
        if switching_opmode_requires_pdo_remapping(&self.opmode, new) {
            // NOTE: this requires a bunch of sdo calls, so take care to call this at cycle start
            self.switch_pdo_config(new)?;
        }

        // NOTE: switch to new opmode, like any state changes, are done at the appropriate place
        // in the sync cycle
        self.opmode = *new;

        Ok(())
    }

    /// Sets new motor setpoint
    pub fn new_motor_setpoint(&mut self, new_sp: Setpoint) {
        self.active_setpoint = new_sp;
    }

    /// Build a script that contains all [`ScriptStep`] required to perform an opmode switch to the
    /// given [`OperationMode`]
    fn get_opmode_switch_script(&mut self, new: &OperationMode) -> Script {
        let new_pdo_cfg = self.pdo_cfg.get_cfg_for_opmode(new);

        let mut start = vec![
            ScriptStep::QuickStop,
            ScriptStep::Nmt {
                cmd: EnterPreOperational,
                expect: PreOperational,
            },
        ];

        // TODO: Compute middle script
        // Should consist of steps in remap_pdo()
        // Everything should be known at compile time, precompute a script for every
        // OMSNodePdoConfig?

        let mut end = vec![
            ScriptStep::Nmt {
                cmd: StartRemoteNode,
                expect: Operational,
            },
            ScriptStep::EnableOperation,
        ];

        Script {
            steps: todo!(),
            cursor: 0,
            started: Instant::now(),
        }
    }

    fn switch_pdo_config(&mut self, new: &OperationMode) -> Result<(), MotorError> {
        let new_pdo_cfg = self.pdo_cfg.get_cfg_for_opmode(new);

        // switch to NMT Pre-OP
        self.request_nmt_command(NmtCommandSpecifier::EnterPreOperational)
            .map_err(|e| MotorError::PdoRemapping(e))?;

        // remap PDO
        for (idx, rpdo) in new_pdo_cfg.rpdo.iter().enumerate() {
            if let Some(rpdo) = rpdo {
                self.remap_pdo(idx as u8, rpdo);
            }
        }
        for (idx, tpdo) in new_pdo_cfg.tpdo.iter().enumerate() {
            if let Some(tpdo) = tpdo {
                self.remap_pdo(idx as u8, tpdo);
            }
        }

        // switch to NMT OP
        self.request_nmt_command(NmtCommandSpecifier::StartRemoteNode)
            .map_err(|e| MotorError::PdoRemapping(e))?;

        self.active_cfg = new_pdo_cfg;
        Ok(())
    }

    pub fn default_parametrisation(&mut self) -> Result<(), MotorError> {
        // switch to NMT Pre-OP
        self.request_nmt_command(NmtCommandSpecifier::EnterPreOperational)
            .map_err(|e| MotorError::PdoRemapping(e))?;

        // Parametrise using SDO
        for sdo_cmd in self.default_parameters {
            if let Err(e) = self.sdo.new_cmd(sdo_cmd.clone()) {
                error!(
                    system = "Cia402Motor",
                    "Failed to default parametrise motor {} - {:?}", self.id, e
                );
                assert!(
                    false,
                    "Failed to default parametrise motor {} - {:?}",
                    self.id, e
                )
            }
        }

        // switch to NMT OP
        self.request_nmt_command(NmtCommandSpecifier::StartRemoteNode)
            .map_err(|e| MotorError::PdoRemapping(e))?;

        Ok(())
    }

    pub fn get_rpdo(&self) -> MotorRpdoSet {
        debug!(system = "Cia402Motor", "get_rpdo");
        let mut out = [None; 4];

        // Iterate active rpdo mappings
        for (num, rpdo) in self.active_cfg.rpdo.iter().enumerate() {
            if let Some(rpdo) = rpdo {
                trace!(system = "Cia402Motor", "get_rpdo - rpdo #{} mapped", num);
                assert!(
                    rpdo.pdo == PdoType::RPDO,
                    "Non-RPDO mapping in self.active_cfg.rpdos.iter()",
                );

                // Iterate RPDO mapping semantics, fetch active value for each
                let semantics = rpdo.get_required_semantics();
                trace!(
                    system = "Cia402Motor",
                    "get_rpdo - required_semantics #{}: {:?}", num, semantics
                );
                let mut values: [Option<PdoValue>; 4] = [const { None }; 4];
                for semantic in semantics.iter().flatten() {
                    let value = self.get_value_from_semantic(semantic);
                    values[num] = Some(value);
                }

                // Encoded active values into a raw rpdo (u64)
                let encoded = rpdo.encode(&values);
                out[num] = Some(encoded);
            } else {
                trace!(system = "Cia402Motor", "get_rpdo - rpdo #{} unmapped", num);
            }
        }

        out
    }

    /// Central work scheduler for the Cia402Motor Controller state machine
    pub fn tick(&mut self, ctx: &mut TickCtx) {
        use ControllerState::*;
        match &mut self.controller_state {
            Idle | Faulted(_) => {}

            Operating => {
                // Check if setpoint is new & requires opmode switch
                let new_opmode = self.active_setpoint.required_opmode();
                // Does this opmode switch warrant a PDO remapping?
                if switching_opmode_requires_pdo_remapping(&self.opmode, &new_opmode) {
                    // Get new opmode PDO mapping
                    let new_pdo_cfg =
                        self.pdo_cfg.get_cfg_for_opmode( &new_opmode);

                    // Get script (==all action steps required) to switch opmode
                    let script = get_pdo_remap_script(&self.id, self.active_cfg,
                        new_pdo_cfg);

                    self.run_script(script, Purpose::OpModeSwitch(new_opmode));
                } else {
                    // No PDO remap required, plain switch -> updated through RPDO later
                    self.opmode = new_opmode;
                }
            }

            Configuring(script, purpose) => {
                let step = &script.steps[script.cursor];
                let done = match step {
                    ScriptStep::QuickStop => self.cia402_state.is_quick_stopped(),
                    ScriptStep::Nmt { expect, .. } => self.nmt_observed == *expect,
                    ScriptStep::Sdo(_) => match self.sdo.poll() {
                        // non-blocking
                        SdoPoll::Done(Ok(_)) => true,
                        SdoPoll::Done(Err(e)) => {
                            self.fault(e.into());
                            return;
                        }
                        SdoPoll::Busy => false,
                        SdoPoll::Idle => false, // waiting for slot, see below
                    },
                    ScriptStep::EnableOperation => {
                        self.cia402_state == Cia402State::OperationEnabled
                    }
                    ScriptStep::DisableOperation => {
                        self.cia402_state == Cia402State::SwitchOnDisabled
                    }
                    ScriptStep::Settle(d) => ctx.now - script.started >= *d,
                };

                if done {
                    script.cursor += 1;
                    if script.cursor == script.steps.len() {
                        // Finish current script
                        self.finish(purpose);
                    } else {
                        // fires the entry action of the new step
                        self.enter_step(ctx);
                    }
                // Check for timeout
                } else if ctx.now - script.started > step.timeout() {
                    self.fault(MotorError::StepTimeout(script.cursor));
                }
            }
        }
    }

    fn enter_step(&mut self, ctx: &mut TickCtx) {
        // The ONLY place an action is performed, exactly once per step.
        match self.current_step() {
            ScriptStep::Nmt { cmd, .. } => { ctx.io.send_nmt(*cmd, &self.id).ok(); self.nmt_commanded = ... }
            ScriptStep::Sdo(cmd) => self.sdo.new_cmd(cmd.clone()),      // sent when the SDO slot allows
            ScriptStep::QuickStop | ScriptStep::EnableOperation | ScriptStep::DisableOperation => {} // via controlword, see below
            ScriptStep::Settle(_) => {}
        }
        self.script.started = ctx.now;
    }

    pub fn on_sync_feedback(&mut self) {
        info!(system = "Cia402Motor", "on_sync_feedback");
    }

    pub fn process_canopen_msg(&mut self, parsed: CanOpenFrame) {
        tracing::trace!(system = "Cia402Motor", "process_canopen_msg({:?})", parsed);
        assert!(
            parsed
                .node_id
                .map(|id| id == self.id.node_id)
                .is_some_and(|out| out),
            "Cia402Motor::process_canopen_msg called msg that contains no node_id: {:?}",
            parsed
        );

        match parsed.msg {
            MessageType::EMCY(emcy) => {
                warn!(system = "Cia402Motor", "Got EMCY msg: {:?}", emcy);
            }

            MessageType::NmtControl(nmt_ctl) => {
                info!(
                    system = "Cia402Motor",
                    "Got NMT CONTROL (setpoint) MSG: {:?}", nmt_ctl
                );
            }

            MessageType::NmtMonitor(nmt_mon) => {
                info!(
                    system = "Cia402Motor",
                    "Got NMT MONITOR (feedback) MSG: {:?}", nmt_mon
                );

                self.nmt = nmt_mon.current_state;
            }

            MessageType::RSDO(s) => {
                info!(
                    system = "Cia402Motor",
                    "Got RSDO {:?}", s
                );
            }

            MessageType::TSDO(s) => {
                info!(
                    system = "Cia402Motor",
                    "Got TSDO {:?}", s
                );
                self.sdo.on_tsdo(s);
            }

            MessageType::PDO(pdo) => {
                // TPDO are sent by the motors, RPDO are received by the motors
                // When processing CANOpen messages, we only care about the motor feedback: TPDO
                if pdo.pdo_type == PdoType::TPDO {
                    assert!(
                        pdo.node_id == self.id.node_id,
                        "Cia402Motor::process_canopen_msg on TPDO with wrong node id {}",
                        pdo.node_id.u8()
                    );

                    assert!(
                        self.active_cfg.tpdo[pdo.num].is_some(),
                        "Cia402Motor::process_canopen_msg on unexpected TRDO num {}",
                        pdo.num
                    );

                    // Attempt to parse TPDO
                    match self.active_cfg.parse_tpdo(&pdo, &mut self.feedback) {
                        Ok(_) => {
                            info!(
                                system = "Cia402Motor",
                                "RPDO parsing & feedback update success for motor: {}",
                                self.id.node_id.u8()
                            );

                            self.motor_state.update_from_motor_feedback(&self.feedback);

                            if self.opmode.is_cyclic_synchronous() {
                                // Mark this RPDO as received for this cycle
                                self.tpdo_rx[pdo.num] = true;
                            }
                        }
                        Err(e) => {
                            error!(system = "Cia402Motor", "Failed to parse RPDO: {}", e)
                        }
                    }
                }
            }

            MessageType::Sync(_) => {
                // Intentionally empty
                trace!(system = "Cia402Motor", "parsed SYNC");
            }
            MessageType::Unknown(msg) => {
                warn!(
                    system = "Cia402Motor",
                    "Unknown CAN dataframe: {:?} - ignoring", msg
                );
            }
        }
    }

    pub fn on_cycle_start(&mut self) {
        // Reset rdpo rx
        self.tpdo_rx = [false; 4];
    }

    // Are all current cycle TPDO received for this motor?
    pub fn all_cycle_tpdo_received(&self) -> bool {
        for (i, pdo_mapping) in self.active_cfg.tpdo.iter().enumerate() {
            if pdo_mapping.is_some() {
                if !self.tpdo_rx[i] {
                    return false;
                }
            }
        }
        true
    }

    fn get_value_from_semantic(&self, semantic: &PdoSemantic) -> PdoValue {
        let out = match semantic {
            PdoSemantic::DeviceType => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::U32(0x0004_0192), // CiA 402 drive hardcoded
            },
            PdoSemantic::DeviceName => {
                let mut out = [0u8; 8];
                out.copy_from_slice(&self.id.device_name.as_bytes()[..8]);
                PdoValue {
                    semantic: semantic.clone(),
                    value: ODValue::VisibleString(out),
                }
            }
            PdoSemantic::Statusword => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::U16(self.statusword.bits()),
            },
            PdoSemantic::Controlword => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::U16(self.controlword.bits()),
            },
            PdoSemantic::ActualPosition => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::I32(self.motor_state.pos.get::<millimeter>() as i32),
            },
            PdoSemantic::ActualVelocity => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::I32(self.motor_state.vel.get::<meter_per_second>() as i32),
            },
            PdoSemantic::ActualTorque => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::I16(self.motor_state.torque.get::<newton_meter>() as i16),
            },
            PdoSemantic::ActualOperationMode => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::I8(self.opmode.into()),
            },
            PdoSemantic::TargetOperationMode => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::I8(self.active_setpoint.required_opmode().into()),
            },
            PdoSemantic::TargetPosition => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::I32(self.target_position),
            },
            PdoSemantic::TargetVelocity => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::I32(self.target_velocity),
            },
            PdoSemantic::TargetTorque => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::I16(self.target_torque),
            },
            PdoSemantic::ProfileVelocity => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::U32(self.limits.profile_velocity),
            },

            PdoSemantic::ProfileAcceleration => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::U32(self.limits.profile_acceleration),
            },
            PdoSemantic::ProfileDecceleration => PdoValue {
                semantic: semantic.clone(),
                value: ODValue::U32(self.limits.profile_decceleration),
            },
            PdoSemantic::Other => {
                warn!(
                    system = "Cia402Motor",
                    "get_value_from_semantic() for PdoSemantic::Other"
                );
                panic!("get_value_from_semantic() for PdoSemantic::Other");
            }
        };

        trace!(
            system = "Cia402Motor",
            "get_value_from_semantic() for {:?} returns {:?}", semantic, out
        );
        out
    }

    pub fn do_e_stop(&mut self) {
        debug!(system = "Cia402Motor", "MOTOR ESTOP TRIGGERED");
        let estop_script = Script::new_starting_now(&script::estop::ESTOP_STEPS);
        self.run_script(estop_script, Purpose::Estop);
    }

    fn run_script(&mut self, script: Script, purpose: Purpose) {
        debug!(
            system = "Cia402Motor",
            "{:?} -> Running script {:?}", purpose, script
        );
        self.controller_state = ControllerState::Configuring(script, purpose);
    }
}

fn switching_opmode_requires_pdo_remapping(old: &OperationMode, new: &OperationMode) -> bool {
    if new.is_cyclic_synchronous() {
        *old != *new
    } else {
        old.is_cyclic_synchronous()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opmode_requires_parametrisation() {
        use OperationMode::*;

        let cases = [
            (Homing, ProfileTorque, false),
            (ProfileTorque, ProfilePosition, false),
            (ProfileTorque, CyclicSynchronousTorque, true),
            (CyclicSynchronousTorque, CyclicSynchronousTorque, false),
            (CyclicSynchronousTorque, CyclicSynchronousPosition, true),
            (CyclicSynchronousTorque, CyclicSynchronousVelocity, true),
            (CyclicSynchronousVelocity, CyclicSynchronousTorque, true),
            (CyclicSynchronousPosition, CyclicSynchronousVelocity, true),
        ];

        for (old, new, switch_expected) in cases {
            if switch_expected {
                assert_eq!(
                    switching_opmode_requires_pdo_remapping(&old, &new),
                    true,
                    "switching opmode from {:?} -> {:?} should require switch",
                    old,
                    new,
                );
            } else {
                assert_eq!(
                    switching_opmode_requires_pdo_remapping(&old, &new),
                    false,
                    "switching opmode from {:?} -> {:?} should require switch",
                    old,
                    new,
                );
            }
        }
    }
}

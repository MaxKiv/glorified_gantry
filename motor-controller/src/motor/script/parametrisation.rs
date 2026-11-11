use crate::{
    canopen::{
        nmt::{
            NmtCommandSpecifier::{EnterPreOperational, StartRemoteNode},
            NmtState::{Operational, PreOperational},
        },
        sdo::{action::SdoAction, manager::SdoCommand},
    },
    cia402::Cia402Identifier,
    motor::script::{
        Purpose, Script,
        ScriptStep::{self, *},
    },
};

/// Get a [Vec<ScriptStep>] containing all actions required to parametrise a [CiA402Motor]
pub fn get_parametrisation_script(id: &Cia402Identifier, parameters: &[SdoAction]) -> Script {
    /// Steps to take before PDO mapping change
    pub const PARAMETRISATION_START_STEPS: [ScriptStep; 2] = [
        QuickStop,
        Nmt {
            cmd: EnterPreOperational,
            expect: PreOperational,
        },
    ];

    /// Steps to take after PDO mapping change
    pub const PARAMETRISATION_END_STEPS: [ScriptStep; 2] = [
        Nmt {
            cmd: StartRemoteNode,
            expect: Operational,
        },
        EnableOperation,
    ];

    let mut steps = Vec::with_capacity(parameters.len() + 4);

    steps.extend(PARAMETRISATION_START_STEPS);
    for action in parameters {
        let cmd = SdoCommand::from_action(id, action);
        steps.push(ScriptStep::Sdo(cmd));
    }
    steps.extend(PARAMETRISATION_END_STEPS);

    Script::from_vec(steps, Purpose::Startup)
}

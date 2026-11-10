pub mod estop;
pub mod opmode_switch;

use std::time::{Duration, Instant};

use crate::{
    canopen::{
        frame::NodeId,
        nmt::{NmtCommandSpecifier, NmtState},
        sdo::manager::SdoCommand,
    },
    oms::OperationMode,
};

#[derive(Debug)]
pub enum Purpose {
    Startup,
    OpModeSwitch(OperationMode),
    Estop,
}

/// Single step in a [`Script`]
/// Performs a single Cia402 related action
#[derive(Debug)]
pub enum ScriptStep {
    QuickStop,
    Nmt {
        cmd: NmtCommandSpecifier,
        expect: NmtState,
    },
    Sdo(SdoCommand),  // one expedited write
    EnableOperation,  // CiA 402 walk to OperationEnabled
    DisableOperation, // startup ends here
    Settle(Duration),
}

impl ScriptStep {
    pub fn sdo_write(node: NodeId, index: u16, sub: u8, data: &[u8]) -> ScriptStep {
        let cmd = SdoCommand::download_raw(node, index, sub, data);
        ScriptStep::Sdo(cmd)
    }
}

/// Collection of chronologically related [`ScriptStep`]
/// A list of actions to perform sequentially
#[derive(Debug)]
pub struct Script {
    pub steps: Box<[ScriptStep]>,
    pub cursor: usize,
    pub started: Instant, // reset by enter_step() when a step is entered
}

impl Script {
    pub fn new(steps: Vec<ScriptStep>) -> Self {
        Self {
            steps: steps.into_boxed_slice(),
            cursor: 0,
            started: Instant::now(),
        }
    }
    pub fn current(&self) -> Option<&ScriptStep> {
        self.steps.get(self.cursor)
    }
    pub fn is_done(&self) -> bool {
        self.cursor >= self.steps.len()
    }
}

use tracing::{info, warn};

use crate::{
    canopen::pdo::message::RawPdoMessage,
    consts::MAX_NODE_ID,
    rt::timekeeper::{CycleTiming, TimeKeeper},
};

#[derive(Debug, PartialEq, Eq)]
pub enum CyclePhase {
    SendingSync,
    WaitingForTpdo,
    ReceivedTpdo,
    SentRpdo,
    SdoWindow,
    CycleEnd,
}

impl CyclePhase {
    pub fn next(&self) -> CyclePhase {
        match self {
            CyclePhase::SendingSync => Self::WaitingForTpdo,
            CyclePhase::WaitingForTpdo => Self::ReceivedTpdo,
            CyclePhase::ReceivedTpdo => Self::SentRpdo,
            CyclePhase::SentRpdo => Self::SdoWindow,
            CyclePhase::SdoWindow => Self::CycleEnd,
            CyclePhase::CycleEnd => Self::SendingSync,
        }
    }
}

pub struct CycleState {
    pub cycle: u64,
    pub phase: CyclePhase,
    pub timekeeper: TimeKeeper,
}

impl CycleState {
    pub fn new() -> Self {
        CycleState {
            cycle: 0u64,
            phase: CyclePhase::SendingSync,
            timekeeper: TimeKeeper::new(),
        }
    }

    pub fn on_feedback_deadline_elapsed(&mut self) {
        assert!(
            self.phase == CyclePhase::WaitingForTpdo,
            "CycleState::on_feedback_deadline_elapsed, but cycle_state.phase = {:?}",
            self.phase
        );

        // TODO: what should the error behavior be when feedback is delayed?
        warn!(
            "TODO figure out feedback delay behavior in CycleState::on_feedback_deadline_elapsed -> restarting cycle..."
        );
        self.transition_cycle_phase(self.phase.next());
        self.transition_cycle_phase(self.phase.next());
        self.transition_cycle_phase(CyclePhase::SdoWindow);
    }

    pub fn get_cycle_timing(&mut self) -> CycleTiming {
        self.timekeeper.get_cycle_timing(self.cycle)
    }

    pub fn transition_cycle_phase(&mut self, to: CyclePhase) {
        assert!(
            to == self.phase.next(),
            "Invalid cycle phase transition from {:?} -> {:?}",
            self.phase,
            to
        );

        self.phase = to;

        // Timekeeping
        match self.phase {
            CyclePhase::SendingSync => {
                info!(phase = "SendingSync", "Start new cycle");
                self.timekeeper.on_sync_cycle_start();
            }
            CyclePhase::WaitingForTpdo => {
                info!(phase = "WaitingForTpdo", "Waiting for TPDO");
                self.timekeeper.on_start_waiting_for_feedback();
            }
            CyclePhase::ReceivedTpdo => {
                info!(phase = "ReceivedTpdo", "Feedback received");
                self.timekeeper.on_receive_feedback();
            }
            CyclePhase::SentRpdo => {
                info!(phase = "SentRpdo", "Setpoints sent");
                self.timekeeper.on_motor_setpoints_sent();
            }
            CyclePhase::SdoWindow => {
                info!(phase = "SdoWindow", "Start SDO window");
                self.timekeeper.on_sdo_window_start();
            }
            CyclePhase::CycleEnd => {
                self.timekeeper.on_cycle_end();
                let ct = self.get_cycle_timing();
                info!(phase = "CycleEnd", "End SYNC Cycle - {:?}", ct);
                self.cycle += 1;
            }
        }
    }
}

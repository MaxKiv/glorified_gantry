use crate::{canopen::pdo::message::RawPdoMessage, consts::MAX_NODE_ID};

#[derive(Debug, PartialEq, Eq)]
pub enum CyclePhase {
    SendingSync,
    WaitingForTpdos,
    SendingRpdoS,
    SdoWindow,
}

pub struct CycleState {
    pub cycle: u64,
    pub phase: CyclePhase,
    pub rpdos_received: [[bool; 4]; MAX_NODE_ID],
}

impl CycleState {
    pub fn new() -> Self {
        CycleState {
            cycle: 0u64,
            phase: CyclePhase::SendingSync,
            rpdos_received: [[false; 4]; MAX_NODE_ID],
        }
    }

    pub fn transition_to(&mut self, target: CyclePhase) {
        match (&self.phase, &target) {
            (CyclePhase::SendingSync, CyclePhase::WaitingForTpdos) => {
                self.phase = CyclePhase::WaitingForTpdos;
            }
            (CyclePhase::WaitingForTpdos, CyclePhase::SendingRpdoS) => {
                self.phase = CyclePhase::SendingRpdoS;
            }
            (CyclePhase::SendingRpdoS, CyclePhase::SdoWindow) => {
                self.phase = CyclePhase::SdoWindow;
            }
            (CyclePhase::SdoWindow, CyclePhase::SendingSync) => {
                self.rpdos_received
                    .iter_mut()
                    .map(|x| x.iter_mut().map(|_| false));
                self.cycle += 1;
                self.phase = CyclePhase::SendingSync;
            }
            _ => panic!(
                "Invalid CycleState transition {:?} -> {:?}",
                self.phase, target
            ),
        }
    }

    pub fn is_all_cycle_feedback_received(&self) -> bool {
        self.rpdos_received.iter().all(|x| x.iter().all(|x| *x))
    }

    pub fn process_rpdo_received(&mut self, pdo: &RawPdoMessage, motor_idx: usize) {
        let node_id = pdo.node_id.get() as usize;
        let received = &mut self.rpdos_received[motor_idx][node_id];

        if *received {
            tracing::warn!(
                "Node {} RPDO {} received more than once this cycle!",
                node_id,
                pdo.num
            );
        } else {
            *received = true;
            tracing::info!("Node {} RPDO {} received!", node_id, pdo.num);
        }
    }
}

pub enum CyclePhase {
    SendingSync,
    WaitingForTpdos,
    SendingRpdoS,
    SyncWindowElapsed,
}

pub struct CycleState {
    pub cycle: u64,
    pub phase: CyclePhase,
    pub received: [bool; 4],
}

impl CycleState {
    pub fn transition_to(&mut self, target: CyclePhase) {
        match (&self.phase, target) {
            (CyclePhase::SendingSync, CyclePhase::WaitingForTpdos) => {
                self.done_sending_sync();
            }
            (CyclePhase::WaitingForTpdos, CyclePhase::SendingRpdoS) => {
                self.done_waiting_for_tpdos();
            }
            (CyclePhase::SendingRpdoS, CyclePhase::SyncWindowElapsed) => {
                self.done_sending_rpdos();
            }
            (CyclePhase::SyncWindowElapsed, CyclePhase::SendingSync) => {
                self.restart_cycle();
            }
            _ => panic!("Invalid CycleState transition"),
        }
    }

    fn done_sending_rpdos(&mut self) {
        self.phase = CyclePhase::SyncWindowElapsed;
    }

    fn done_waiting_for_tpdos(&mut self) {
        self.phase = CyclePhase::SendingRpdoS;
    }

    fn done_sending_sync(&mut self) {
        self.phase = CyclePhase::WaitingForTpdos;
    }

    fn restart_cycle(&mut self) {
        for rx in &mut self.received {
            *rx = false;
        }
        self.cycle += 1;
        self.phase = CyclePhase::SendingSync;
    }
}

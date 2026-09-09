#[derive(Debug)]
pub struct NmtControlMessage {
    pub node_id: NodeId,
    pub requested_state: NmtState,
}

#[derive(Debug)]
pub struct NmtMonitorMessage {
    pub node_id: NodeId,
    pub current_state: NmtState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NmtState {
    Bootup,
    Stopped,
    PreOperational,
    Operational,
}
impl NmtState {
    fn from_nmt_command_frame(frame_data: &[u8]) -> Self {
        match frame_data[0] {
            0x01 => NmtState::Operational,
            0x02 => NmtState::Stopped,
            0x80 => NmtState::PreOperational,
            _ => NmtState::PreOperational,
        }
    }

    fn from_node_monitoring_frame(frame_data: &[u8]) -> Self {
        match frame_data[0] {
            0x00 => NmtState::Bootup,
            0x04 => NmtState::Stopped,
            0x05 => NmtState::Operational,
            0x7F => NmtState::PreOperational,
            _ => NmtState::PreOperational,
        }
    }
}

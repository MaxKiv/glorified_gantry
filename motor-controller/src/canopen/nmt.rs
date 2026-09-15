use socketcan::{CanFrame, Frame};

use crate::{canopen::frame::NodeId, cia402::Cia402Identifier};

/// Represents the NMT command specifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NmtCommandSpecifier {
    /// Start the remote node.
    StartRemoteNode = 0x01,
    /// Stop the remote node.
    StopRemoteNode = 0x02,
    /// Enter pre-operational state.
    EnterPreOperational = 0x80,
    /// Reset the node.
    ResetNode = 0x81,
    /// Reset the communication.
    ResetCommunication = 0x82,
}

impl TryFrom<u8> for NmtCommandSpecifier {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::StartRemoteNode),
            0x02 => Ok(Self::StopRemoteNode),
            0x80 => Ok(Self::EnterPreOperational),
            0x81 => Ok(Self::ResetNode),
            0x82 => Ok(Self::ResetCommunication),
            other => Err(other),
        }
    }
}

impl NmtCommandSpecifier {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug)]
pub struct NmtControlMessage {
    pub node_id: NodeId,
    pub requested_command: NmtCommandSpecifier,
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

    pub fn from_cmd(cmd: &NmtCommandSpecifier) -> Self {
        match cmd {
            NmtCommandSpecifier::StartRemoteNode => NmtState::Operational,
            NmtCommandSpecifier::StopRemoteNode => NmtState::Stopped,
            NmtCommandSpecifier::EnterPreOperational => NmtState::PreOperational,
            NmtCommandSpecifier::ResetNode => NmtState::Bootup,
            NmtCommandSpecifier::ResetCommunication => NmtState::Bootup,
        }
    }
}

pub struct NmtFrame {
    pub inner: CanFrame,
}

impl NmtFrame {
    pub fn new_cmd_to_node(cmd: NmtCommandSpecifier, node: &Cia402Identifier) -> Self {
        const NMT_COB_ID: u32 = 0x0;
        let inner: CanFrame = CanFrame::from_raw_id(NMT_COB_ID, &[cmd.as_u8(), node.node_id.u8()])
            .expect("failed to construct NMT frame");

        Self { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nmt_cmd_as_u8() {
        let cases = [
            (NmtCommandSpecifier::ResetNode, 0x01),
            (NmtCommandSpecifier::StopRemoteNode, 0x02),
            (NmtCommandSpecifier::EnterPreOperational, 0x80),
            (NmtCommandSpecifier::ResetNode, 0x81),
            (NmtCommandSpecifier::ResetCommunication, 0x82),
        ];

        for (cmds, value) in cases {
            assert_eq!(
                cmds.as_u8(),
                value,
                "{:?} as_u8() should return {} but didn't",
                cmds,
                value
            );
        }
    }
}

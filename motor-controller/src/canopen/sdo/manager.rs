use std::time::{Duration, Instant};

use tracing::{debug, error, info, warn};

use crate::{
    canopen::{
        CanOpen, CanOpenError,
        frame::NodeId,
        od::{entry::ODEntry, value::ODValue},
        sdo::{
            NormalSdoDownload, RawSdoDownload, SdoDownload, SdoDownloadConfirmed, SdoResponse,
            SdoUpload, SdoUploadResult, action::SdoAction,
        },
    },
    cia402::Cia402Identifier,
    fifo::{Fifo, error::FifoError},
};

const SDO_EVENT_Q_SIZE: usize = 64;
const SDO_CMD_Q_SIZE: usize = 16;
const MAX_EVENTS_PER_TICK: usize = 16;
const SDO_TIMEOUT: Duration = Duration::from_millis(500);

pub struct SdoManager {
    events: Fifo<SdoResponse, SDO_EVENT_Q_SIZE>,
    cmd_queue: Fifo<SdoCommand, SDO_CMD_Q_SIZE>,
    current: Option<InFlight>, // Currently active SdoCommand + deadline
    result: Option<Result<SdoOutcome, SdoFailure>>, // Consumed in poll()
}

#[derive(Debug, thiserror::Error)]
pub enum SdoManagerError {
    #[error("Attempted to enqueue command into full queue")]
    CommandQueueFull,
}

struct InFlight {
    cmd: SdoCommand,
    deadline: Instant,
}

pub enum SdoPoll {
    Idle,                                 // nothing queued or in flight
    Busy,                                 // queued or awaiting response
    Done(Result<SdoOutcome, SdoFailure>), // delivered exactly once
}

#[derive(Debug)]
pub enum SdoFailure {
    Abort(u32),
    Timeout,
    Tx(CanOpenError),
}

// upload payload, None for download
pub struct SdoOutcome {
    pub data: Option<[u8; 4]>,
}

#[derive(Debug, Clone)]
pub struct SdoCommand {
    request: SdoManagerRequest,
    expected_answer: SdoResponse,
}

impl SdoCommand {
    pub fn upload(node: &'static Cia402Identifier, od_entry: &'static ODEntry) -> Self {
        let request = SdoManagerRequest::Upload(SdoUpload {
            node,
            od_entry,
            result: None,
        });
        let result = SdoUploadResult::new_from_od_entry(node.node_id, od_entry);
        let expected_answer = SdoResponse::UploadConfirm(result);

        SdoCommand {
            request,
            expected_answer,
        }
    }

    pub fn download(
        node: &'static Cia402Identifier,
        od_entry: &'static ODEntry,
        value: ODValue,
    ) -> Self {
        let request = SdoManagerRequest::Download(SdoDownload::Normal(NormalSdoDownload {
            node,
            od_entry,
            result: None,
            value,
        }));
        let result = SdoDownloadConfirmed::new_from_od_entry(node.node_id, od_entry);
        let expected_answer = SdoResponse::DownloadConfirm(result);

        SdoCommand {
            request,
            expected_answer,
        }
    }

    pub fn download_raw(node: NodeId, index: u16, sub: u8, data: &[u8], len: usize) -> Self {
        let mut dat = [0u8; 8];
        dat[..len].copy_from_slice(data);
        let request = SdoManagerRequest::Download(SdoDownload::Raw(RawSdoDownload {
            node,
            index,
            sub,
            data: dat,
            len,
            result: None,
        }));
        let expected_answer = SdoResponse::DownloadConfirm(SdoDownloadConfirmed {
            from: node,
            index,
            sub,
        });

        SdoCommand {
            request,
            expected_answer,
        }
    }

    pub fn from_action(node: &'static Cia402Identifier, action: &SdoAction) -> Self {
        match action {
            SdoAction::Download { entry, data } => {
                // TODO: hacky?
                let mut arr = [0u8; 8];
                arr[..data.len()].copy_from_slice(data);
                let raw = u64::from_le_bytes(arr);
                let data = entry.semantic.to_odvalue(raw);
                SdoCommand::download(node, entry, data)
            }
            SdoAction::Upload { entry } => SdoCommand::upload(node, entry),
        }
    }

    pub fn bullshit() -> Self {
        Self {
            request: SdoManagerRequest::Upload(SdoUpload {
                node: &crate::cia402::BS_NODE,
                od_entry: &crate::canopen::od::DEVICE_TYPE,
                result: None,
            }),
            expected_answer: SdoResponse::Error(super::SdoError {
                from: crate::cia402::BS_NODE.node_id,
                index: 0,
                sub: 0,
                code: 0,
            }),
        }
    }

    fn key(&self) -> (NodeId, u16, u8) {
        match self.request {
            SdoManagerRequest::Upload(SdoUpload { node, od_entry, .. })
            | SdoManagerRequest::Download(SdoDownload::Normal(NormalSdoDownload {
                node,
                od_entry,
                ..
            })) => (node.node_id.clone(), od_entry.index, od_entry.sub),
            SdoManagerRequest::Download(SdoDownload::Raw(RawSdoDownload {
                node,
                index,
                sub,
                ..
            })) => (node.clone(), index, sub),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SdoManagerRequest {
    Upload(SdoUpload),
    Download(SdoDownload),
}

#[derive(Debug, Default)]
pub enum SdoManagerState {
    #[default]
    Idle,
    WaitingForResponse {
        expected: SdoResponse,
        deadline: Instant,
    },
    Failed(SdoCommand),
}

impl SdoManager {
    pub fn new() -> Self {
        SdoManager {
            events: Fifo::new(),
            cmd_queue: Fifo::new(),
            current: None,
            result: None,
        }
    }

    /// Ask the SDO Manager to perform a SDO call -> enqueues new SdoCommand
    pub fn new_cmd(&mut self, cmd: SdoCommand) -> Result<(), SdoManagerError> {
        self.cmd_queue
            .push(cmd)
            .map_err(|_| SdoManagerError::CommandQueueFull)?;
        Ok(())
    }

    /// Enqueues the Sdo Response for later consumption in tick()
    pub fn on_tsdo(&mut self, response: SdoResponse) {
        if let Err(err) = self.events.push(response) {
            match err {
                FifoError::Full(event) => {
                    let discarded_event = self.events.pop().expect("");
                    error!(
                        system = "SdoManager",
                        "on_tsdo fail: SDO Fifo queue full - discarding last event in queue: {:?}",
                        discarded_event
                    );
                    let _ = self.events.push(event);
                }
                FifoError::Empty => {
                    error!(
                        system = "SdoManager",
                        "INVALID_STATE: on_tsdo fail: SDO Fifo queue empty - ignoring",
                    );
                }
            }
        }
    }

    /// SDO manager state machine work scheduler
    pub fn tick(&mut self, now: Instant, can: &mut CanOpen, slot_free: bool) {
        // Is there a current action
        match self.current.take() {
            None => {
                if slot_free && let Ok(cmd) = self.cmd_queue.pop() {
                    // drop stale frames
                    while self.events.pop().is_ok() {}

                    match can.send_sdo_request(&cmd.request) {
                        Ok(()) => {
                            self.current = Some(InFlight {
                                cmd,
                                deadline: now + SDO_TIMEOUT,
                            })
                        }
                        Err(e) => {
                            error!(
                                system = "SdoManager",
                                "send_sdo_request() returned error: {:?}", e
                            );
                            self.result = Some(Err(SdoFailure::Tx(e)));
                        }
                    }
                }
            }

            Some(current) => {
                // Drain a bounded number of events
                for _ in 0..MAX_EVENTS_PER_TICK {
                    let Ok(resp) = self.events.pop() else { break };
                    // match (node, index, sub)
                    if resp.key() != current.cmd.key() {
                        warn!("ignoring unrelated TSDO {resp:?}");
                        continue;
                    }
                    self.result = Some(match resp {
                        SdoResponse::Error(e) => Err(SdoFailure::Abort(e.code)),
                        SdoResponse::UploadConfirm(u) => Ok(SdoOutcome { data: Some(u.data) }),
                        SdoResponse::DownloadConfirm(_) => Ok(SdoOutcome { data: None }),
                    });
                    return;
                }
                if now >= current.deadline {
                    // deadline expired, report failure
                    self.result = Some(Err(SdoFailure::Timeout));
                } else {
                    // keep waiting
                    self.current = Some(current);
                }
            }
        }
    }

    /// Non-blocking state polling
    pub fn poll(&mut self) -> SdoPoll {
        if let Some(res) = self.result.take() {
            // consume-once: next call sees Busy/Idle
            return SdoPoll::Done(res);
        }
        if self.current.is_some() || !self.cmd_queue.is_empty() {
            SdoPoll::Busy
        } else {
            SdoPoll::Idle
        }
    }
}

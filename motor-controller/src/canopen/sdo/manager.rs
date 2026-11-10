use std::time::{Duration, Instant};

use tracing::{debug, error, info, warn};

use crate::{
    canopen::{
        CanOpen,
        frame::NodeId,
        od::{entry::ODEntry, value::ODValue},
        sdo::{
            NormalSdoDownload, RawSdoDownload, SdoDownload, SdoDownloadConfirmed, SdoRequest,
            SdoResponse, SdoUpload, SdoUploadResult,
        },
    },
    cia402::Cia402Identifier,
    fifo::{Fifo, error::FifoError},
};

const SDO_EVENT_Q_SIZE: usize = 64;
const SDO_CMD_Q_SIZE: usize = 16;
const MAX_EVENTS_PER_TICK: usize = 16;
const SDO_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, thiserror::Error)]
pub enum SdoManagerError {
    #[error("Attempted to enqueue command into full queue")]
    CommandQueueFull,
}

pub struct SdoManager {
    canopen: CanOpen,
    state: SdoManagerState,
    events: Fifo<SdoManagerEvent, SDO_EVENT_Q_SIZE>,
    cmd_queue: Fifo<SdoCommand, SDO_CMD_Q_SIZE>,
    current_cmd: Option<SdoCommand>,
}

/// CANOpen bus events relevant to SdoManager
#[derive(Debug)]
pub enum SdoManagerEvent {
    Request(SdoRequest),
    Response(SdoResponse),
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

    pub fn download_raw(node: NodeId, index: u16, sub: u8, data: &[u8]) -> Self {
        let mut dat = [0u8; 8];
        dat.copy_from_slice(data);
        let request = SdoManagerRequest::Download(SdoDownload::Raw(RawSdoDownload {
            node,
            index,
            sub,
            data: dat,
            result: None,
        }));
        let expected_answer = SdoResponse::DownloadConfirm(SdoDownloadConfirmed {
            from: node,
            index,
            sub_index: sub,
        });

        SdoCommand {
            request,
            expected_answer,
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
    pub fn new(canopen: CanOpen) -> Self {
        SdoManager {
            canopen,
            state: SdoManagerState::default(),
            events: Fifo::new(),
            cmd_queue: Fifo::new(),
            current_cmd: None,
        }
    }

    /// Ask the SDO Manager to perform a SDO call
    pub fn new_cmd(&mut self, cmd: SdoCommand) -> Result<(), SdoManagerError> {
        self.cmd_queue
            .push(cmd)
            .map_err(|_| SdoManagerError::CommandQueueFull)?;
        Ok(())
    }

    pub fn on_tsdo(&mut self, sdo: SdoResponse) {
        if let Err(err) = self.events.push(SdoManagerEvent::Response(sdo)) {
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

    /// Progress SDO manager state machine
    /// NOTE: keep this quick, this is called every RT loop
    pub fn tick(&mut self) {
        match self.state {
            // What to do in idle
            SdoManagerState::Idle => {
                self.try_start_new_cmd();
            }

            // What to do when waiting for up/download response
            SdoManagerState::WaitingForResponse { expected, deadline } => {
                // SDO Manager waiting for response, check if event is response
                // Consumes a few even per tick
                self.on_waiting_for_response(&expected, &deadline);
            }

            SdoManagerState::Failed(ref sdo_manager_request) => {
                error!(
                    system = "SdoManager",
                    "Transitioned to FAILED state due to recent failure of {:?}, resetting to idle",
                    sdo_manager_request
                );
                self.state = SdoManagerState::Idle;
            }
        };
    }

    /// Queues a new SDO
    fn queue_sdo(&mut self, cmd: SdoCommand) {
        let request = cmd.request.clone();
        let expected = cmd.expected_answer.clone();
        if let Err(_) = self.canopen.queue_sdo(request.clone()) {
            error!(
                system = "SdoManager",
                "Transitioned to FAILED state due to recent failure of {:?}, resetting to idle",
                request
            );
            self.state = SdoManagerState::Failed(cmd);
        }

        let deadline = Instant::now() + SDO_TIMEOUT;
        self.state = SdoManagerState::WaitingForResponse { expected, deadline }
    }

    /// Checks the SDO event queue if current command completed sucessfully
    fn check_cmd_success(&mut self, node: &'static Cia402Identifier, od_entry: &'static ODEntry) {
        // Drain MAX_EVENTS_PER_TICK events per tick
        for _ in 0..MAX_EVENTS_PER_TICK {
            if let Ok(event) = self.events.pop() {
                match event {
                    SdoManagerEvent::Request(request) => {
                        // SDO manager did request up/download, check if the correct RSDO appeared on bus
                        if request.to == node.node_id {
                            if let Some(ref value) = request.value {
                                if *value == *od_entry {
                                    // Correct RSDO appeared on bus, nice!
                                    if let Some(current_cmd) = &self.current_cmd {
                                        let deadline = Instant::now() + SDO_TIMEOUT;
                                        self.state = SdoManagerState::WaitingForResponse {
                                            expected: current_cmd.expected_answer,
                                            deadline,
                                        };
                                        info!(
                                            "RSDO for {:?} detected, waiting for TSDO {:?}",
                                            self.state, current_cmd.expected_answer
                                        );

                                        // Process rest of events another tick
                                        break;
                                    } else {
                                        error!(
                                            system = "SdoManager",
                                            "In State {:?} but No current_cmd, logical bug",
                                            self.state
                                        );
                                    }
                                }
                            }
                        }
                        error!(
                            system = "SdoManager",
                            "Incorrect RSDO detected (node {}, value {:?}), expecting {:?}",
                            request.to.u8(),
                            request.value,
                            self.state
                        );
                    }
                    SdoManagerEvent::Response(_) => {
                        error!(
                            system = "SdoManager",
                            "Unexpected event {:?} for state {:?}", event, self.state
                        );
                    }
                }
            } else {
                break;
            }
        }
    }

    fn on_waiting_for_response(&mut self, expected: &SdoResponse, deadline: &Instant) {
        // Check for deadline expiration
        if *deadline <= Instant::now() {
            error!(
                system = "SdoManager",
                "SDO Deadline expired for {:?}!", self.current_cmd
            );
            // deadline expired
            let cmd = if let Some(current_cmd) = self.current_cmd.clone() {
                current_cmd
            } else {
                error!(
                    system = "SdoManager",
                    "SDO Deadline expired, but no current command, invalid state"
                );
                SdoCommand::bullshit()
            };
            self.state = SdoManagerState::Failed(cmd);
            return;
        }

        // Deadline not expired
        // Drain MAX_EVENTS_PER_TICK events per tick
        for _ in 0..MAX_EVENTS_PER_TICK {
            if self.check_if_next_event_matches(expected) {
                debug!(
                    system = "SdoManager",
                    "Witnessed expected event - enqueuing next event"
                );

                self.try_start_new_cmd();
                break;
            }
        }

        warn!(
            system = "SdoManager",
            "Still waiting for {:?} - retrying next tick()", self.state
        );
    }

    fn completed_cmd(&mut self) {
        self.state = SdoManagerState::Idle;
        self.current_cmd = None;
    }

    fn check_if_next_event_matches(&mut self, expected: &SdoResponse) -> bool {
        if let Ok(event) = self.events.pop() {
            match event {
                SdoManagerEvent::Request(_) => {
                    error!(
                        system = "SdoManager",
                        "Unexpected event {:?} for state {:?}", event, self.state
                    );
                    panic!("Unexpected event {:?} for state {:?}", event, self.state);
                }

                SdoManagerEvent::Response(response) => {
                    // Event is response
                    match (expected, response) {
                        (
                            SdoResponse::DownloadConfirm(rx_download),
                            SdoResponse::DownloadConfirm(expected_download),
                        ) => {
                            if *rx_download == expected_download {
                                info!(system = "SdoManager", "TSDO rx success");
                                self.completed_cmd();
                                // Process the rest of the events another tick
                                return true;
                            } else {
                                warn!(
                                    system = "SdoManager",
                                    "TSDO rx fail: {:?} != {:?}", rx_download, expected_download
                                );
                                panic!("TSDO rx fail: {:?} != {:?}", rx_download, expected_download)
                            }
                        }
                        (
                            SdoResponse::UploadConfirm(rx_upload),
                            SdoResponse::UploadConfirm(expected_upload),
                        ) => {
                            if *rx_upload == expected_upload {
                                info!(system = "SdoManager", "TSDO rx success");
                                self.completed_cmd();
                                // Process the rest of the events another tick
                                return true;
                            } else {
                                error!(
                                    system = "SdoManager",
                                    "TSDO rx fail: {:?} != {:?}", rx_upload, expected_upload
                                );
                                panic!("TSDO rx fail: {:?} != {:?}", rx_upload, expected_upload)
                            }
                        }
                        _ => {
                            error!(
                                system = "SdoManager",
                                "Wrong SDO response, expected {:?} got {:?}", expected, response
                            );
                            panic!(
                                "Wrong SDO response, expected {:?} got {:?}",
                                expected, response
                            )
                        }
                    }
                }
            };
        }

        false
    }

    /// Start new SDO transaction,
    /// if that works -> sets state to WaitingForResponse
    /// if no transaction requested > sets state to Idle
    fn try_start_new_cmd(&mut self) {
        if self.current_cmd.is_none()
            && let Ok(request) = self.cmd_queue.pop()
        {
            // Ready to start new command
            self.current_cmd = Some(request.clone());
            // Queue SDO with CANOpen system, this will send out the SDO whenever RT scheduling
            // deems it appropriate
            self.queue_sdo(request);
        } else {
            self.state = SdoManagerState::Idle;
        }
    }
}

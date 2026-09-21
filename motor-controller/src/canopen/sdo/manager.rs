use std::time::{Duration, Instant};

use tracing::{error, info, warn};

use crate::{
    canopen::{
        CanOpen,
        od::{entry::ODEntry, value::ODValue},
        sdo::{
            SdoDownload, SdoDownloadConfirmed, SdoRequest, SdoResponse, SdoUpload, SdoUploadResult,
        },
    },
    cia402::Cia402Identifier,
    fifo::{Fifo, error::FifoError},
};

const SDO_EVENT_Q_SIZE: usize = 64;
const SDO_CMD_Q_SIZE: usize = 64;
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

    pub fn upload_raw(node: &'static Cia402Identifier) -> Self {
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
        let request = SdoManagerRequest::Download(SdoDownload {
            node,
            od_entry,
            result: None,
            value,
        });
        let result = SdoDownloadConfirmed::new_from_od_entry(node.node_id, od_entry);
        let expected_answer = SdoResponse::DownloadConfirm(result);

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
                sub_index: 0,
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
    Uploading(SdoUpload),
    Downloading(SdoDownload),
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

    pub fn on_rsdo(&mut self, sdo: SdoRequest) -> Result<usize, FifoError<SdoManagerEvent>> {
        self.events.push(SdoManagerEvent::Request(sdo))
    }

    pub fn on_tsdo(&mut self, sdo: SdoResponse) -> Result<usize, FifoError<SdoManagerEvent>> {
        self.events.push(SdoManagerEvent::Response(sdo))
    }

    /// Progress SDO manager state machine
    /// NOTE: keep this quick, this is called every RT loop
    pub fn tick(&mut self) {
        match self.state {
            // What to do in idle
            SdoManagerState::Idle => {
                if self.current_cmd.is_none()
                    && let Ok(req) = self.cmd_queue.pop()
                {
                    // Ready to start new command
                    self.current_cmd = Some(req.clone());
                    self.send_cmd(req);
                }
            }

            // What to do when an up/download recently started
            SdoManagerState::Uploading(SdoUpload { node, od_entry, .. })
            | SdoManagerState::Downloading(SdoDownload { node, od_entry, .. }) => {
                // SDO Manager in Down/upload state, check if event is request
                // Consumes a few even per tick
                self.on_cmd_started(node, od_entry);
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

    fn send_cmd(&mut self, SdoCommand { request, .. }: SdoCommand) {
        match request {
            SdoManagerRequest::Upload(sdo) => match self.canopen.send_sdo_upload(&sdo) {
                Ok(_) => self.state = SdoManagerState::Uploading(sdo),
                Err(e) => {
                    error!(system = "SdoManager", "RSDO upload tx fail: {:?}", e);
                }
            },
            SdoManagerRequest::Download(sdo) => match self.canopen.send_sdo_download(&sdo) {
                Ok(_) => self.state = SdoManagerState::Downloading(sdo),
                Err(e) => {
                    error!(system = "SdoManager", "RSDO download tx fail: {:?}", e);
                }
            },
        };
    }

    fn on_cmd_started(&mut self, node: &'static Cia402Identifier, od_entry: &'static ODEntry) {
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
            let cmd = self.current_cmd.unwrap_or_else(|| {
                error!(
                    system = "SdoManager",
                    "SDO Deadline expired, but no current command, invalid state"
                );
                SdoCommand::bullshit()
            });
            self.state = SdoManagerState::Failed(cmd);
            return;
        }

        // Drain MAX_EVENTS_PER_TICK events per tick
        for _ in 0..MAX_EVENTS_PER_TICK {
            if let Ok(event) = self.events.pop() {
                match event {
                    SdoManagerEvent::Request(_) => {
                        warn!(
                            system = "SdoManager",
                            "Unexpected event {:?} for state {:?}", event, self.state
                        );
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
                                    break;
                                } else {
                                    warn!(
                                        system = "SdoManager",
                                        "TSDO rx fail: {:?} != {:?}",
                                        rx_download,
                                        expected_download
                                    );
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
                                    break;
                                } else {
                                    warn!(
                                        system = "SdoManager",
                                        "TSDO rx fail: {:?} != {:?}", rx_upload, expected_upload
                                    );
                                }
                            }
                            _ => {
                                warn!(
                                    system = "SdoManager",
                                    "Wrong SDO response, expected {:?} got {:?}",
                                    expected,
                                    response
                                );
                            }
                        }
                    }
                };
            } else {
                break;
            }
        }
    }

    fn completed_cmd(&mut self) {
        self.state = SdoManagerState::Idle;
        self.current_cmd = None;
    }
}

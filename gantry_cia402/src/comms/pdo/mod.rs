pub mod manager;
pub mod mapping;

use std::thread::JoinHandle;

/// PDO based Cia402Transport impl for oze-canopen
use oze_canopen::interface::CanOpenInterface;

use crate::comms::transport::Cia402Transport;

pub struct Pdo {
    canopen: CanOpenInterface,
    handle: JoinHandle<()>,
}

impl Pdo {
    pub fn new() -> Self {
        task::spawn()
    }
}

#[async_trait::async_trait]
impl Cia402Transport for Pdo {
    async fn write_controlword(&self, cw: u16) -> anyhow::Result<()> {}

    async fn read_statusword(&self) -> anyhow::Result<u16> {
        todo!()
    }

    async fn write_operation_mode(&self, mode: u8) -> anyhow::Result<()> {
        todo!()
    }

    async fn read_operation_mode_display(&self) -> anyhow::Result<u8> {
        todo!()
    }
}

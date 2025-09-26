use std::sync::Arc;

use tokio::{
    select,
    sync::{Mutex, mpsc::Receiver},
};
use tracing::*;

use crate::{
    comms::pdo::Pdo,
    error::DriveError,
    od::{
        bitmask::BitMask,
        oms::{DEFAULT_OPERATIONAL_MODE_SETPOINT, Setpoint},
        state::Cia402State,
    },
};

pub type ControlWord = u16;
const DEFAULT_UPDATE: Update = Update {
    controlword: 0,
    setpoint: Some(DEFAULT_OPERATIONAL_MODE_SETPOINT),
};

#[derive(Debug, Clone)]
struct Update {
    controlword: ControlWord,
    setpoint: Option<Setpoint>,
}

/// Responsible for all CANopen communication to the drive
/// Receives updates from the Cia402 state machine and operational mode specific handler
/// It encodes these changes into the appropriate controlword bits or OD object
/// It then sends these changes out on the CANopen bus using the accessor
pub async fn publish_updates(
    mut accessor: Arc<Mutex<Pdo>>,
    mut setpoint_rx: Receiver<Setpoint>,
    mut state_rx: Receiver<Cia402State>,
) {
    let mut update = DEFAULT_UPDATE;

    trace!("0. Send default update on boot: {update:?}");
    if let Err(e) = write_update(&mut accessor, update.clone())
        .instrument(tracing::info_span!("writing update to device"))
        .await
    {
        error!("failed to write default update {update:?} on boot: {e:?}");
    }

    loop {
        let mut controlword_mask = BitMask { set: 0, clear: 0 };
        update.setpoint = None;

        trace!("1. Wait for a relevant change that requires a controlword or OD object update");
        select! {
            maybe_state = state_rx.recv() => {
                if let Some(new_state) = maybe_state {
                    trace!("state change received: {new_state:?}, updating controlword");
                    controlword_mask = BitMask::get_controlword_mask_for_state(&new_state);
                }
            }
            maybe_oms = setpoint_rx.recv() => {
                if let Some(oms_setpoint) = maybe_oms {
                    trace!("operational mode specific change received: {oms_setpoint:?}, updating controlword and setpoint");
                    controlword_mask = BitMask::get_controlword_mask_for_oms_setpoint(&oms_setpoint);
                    update.setpoint = Some(oms_setpoint);
                }
            }
        }

        trace!("2. Transform controlword to reflect the new mask: {controlword_mask:?}");
        update.controlword = BitMask::apply_controlword_mask(controlword_mask, update.controlword);

        trace!(
            "3. Sent updated controlword: {} to the motor",
            update.controlword
        );
        if let Err(e) = write_update(&mut accessor, update.clone())
            .instrument(tracing::info_span!("writing update to device"))
            .await
        {
            error!("failed to write update: {e:?}");
        }
    }
}

pub async fn write_update(
    accessor: &mut Arc<Mutex<Pdo>>,
    update: Update,
) -> Result<(), DriveError> {
    trace!("1. writing updated setpoint {:?}", update.setpoint);
    match update.setpoint {
        Some(Setpoint::ProfilePosition(position_setpoint)) => {
            accessor
                .lock()
                .await
                .write_position_setpoint(position_setpoint)
                .await?;
        }
        Some(Setpoint::ProfileVelocity(velocity_setpoint)) => {
            accessor
                .lock()
                .await
                .write_velocity_setpoint(velocity_setpoint)
                .await?;
        }
        Some(Setpoint::ProfileTorque(torque_setpoint)) => {
            accessor
                .lock()
                .await
                .write_torque_setpoint(torque_setpoint)
                .await?;
        }
        _ => {}
    }

    trace!("2. writing updated controlword {}", update.controlword);
    accessor
        .lock()
        .await
        .write_controlword(update.controlword)
        .await?;

    Ok(())
}

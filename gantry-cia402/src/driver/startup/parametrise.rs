use std::sync::Arc;

use anyhow::Result;
use oze_canopen::sdo_client::SdoClient;
use tokio::sync::Mutex;
use tracing::*;

use crate::{
    comms::sdo::{SDO_PROCESS_DURATION, SdoAction},
    driver::{
        identifier::{
            Cia402Identifier, CiaProfileNumber,
            MotorType,
        },
        startup::params::{DEVICE_NAME_ACTION, DEVICE_TYPE_ACTION},
    },
    error::InitialisationError,
};

/// Parametrize the motor at given node id
/// parametrisation is the process of setting important parameters like
/// maximum velocity or torque to known values at boot
/// The motor usually does not commit these changes to non-volatile memory,
/// so this has to run on every new boot cycle of the device
pub async fn parametrise_motor(
    identifier: Cia402Identifier,
    parameters: &[SdoAction<'_>],
    sdo: Arc<Mutex<SdoClient>>,
) -> Result<(), InitialisationError> {
    // First check if the motor we are talking to is the one we expect
    check_device_type_is_as_expected(&identifier, sdo.clone()).await?;

    trace!(
        "starting parametrisation of Motor with node id {}",
        identifier.node_id
    );

    // parametrisation is done through a series of SDO calls, perform these in order
    for action in parameters {
        trace!(
            "parametrizing node id {} with: {action:?}",
            identifier.node_id
        );

        match action.run_on_sdo_client(sdo.clone()).await {
            Ok(sdo_transaction) => {
                info!("SDO result: {:?}", sdo_transaction);

                if let crate::comms::sdo::SdoResult::Error(err) = sdo_transaction.result {
                    error!(
                        "SDO Error {}, during parametrisation of node {} with {action:?}",
                        err, identifier.node_id
                    );
                    return Err(InitialisationError::ParametrisationError(
                        identifier.clone(),
                    ));
                }
            }
            Err(err) => error!(
                "Error while parametrizing node id {}: {err}",
                identifier.node_id
            ),
        };

        // Shit synchronisation
        tokio::time::sleep(SDO_PROCESS_DURATION).await;
    }

    Ok(())
}

async fn check_device_type_is_as_expected(
    identifier: &Cia402Identifier,
    sdo: Arc<Mutex<SdoClient>>,
) -> Result<(), InitialisationError> {
    trace!(
        "Checking Device Type, Motor Type & Device Name before starting parametrisation of Motor {:?}",
        identifier
    );

    // Check device type is expected
    let sdo_transaction = DEVICE_TYPE_ACTION
        .run_on_sdo_client(sdo.clone())
        .await
        .map_err(|_| {
            InitialisationError::ParametrisationCommunicationFailure(identifier.clone())
        })?;
    let crate::comms::sdo::SdoResult::Data(vec) = sdo_transaction.result else {
        return Err(InitialisationError::ParametrisationCommunicationFailure(
            identifier.clone(),
        ));
    };
    // Check device profile number
    let device_profile_number: u16 = u16::from_le_bytes(vec[..2].try_into().map_err(|_| {
        InitialisationError::ParametrisationCommunicationFailure(identifier.clone())
    })?);
    let device_profile: CiaProfileNumber = device_profile_number.try_into().map_err(|e| {
        InitialisationError::ParametrisationInvalidCiaProfileNumber(e, identifier.clone())
    })?;
    if device_profile != identifier.device_profile_number {
        return Err(InitialisationError::ParametrisationWrongCiaProfileNumber(
            device_profile,
            identifier.clone(),
        ));
    }
    // check motor type
    let motor_type_number: u16 = u16::from_le_bytes(vec[2..].try_into().map_err(|_| {
        InitialisationError::ParametrisationCommunicationFailure(identifier.clone())
    })?);
    let motor_type: MotorType = motor_type_number
        .try_into()
        .map_err(|e| InitialisationError::ParametrisationInvalidMotorType(e, identifier.clone()))?;
    if motor_type != identifier.motor_type {
        return Err(InitialisationError::ParametrisationWrongMotorType(
            motor_type,
            identifier.clone(),
        ));
    }

    // Check device name is expected
    let sdo_transaction = DEVICE_NAME_ACTION
        .run_on_sdo_client(sdo.clone())
        .await
        .map_err(|_| {
            InitialisationError::ParametrisationCommunicationFailure(identifier.clone())
        })?;
    let crate::comms::sdo::SdoResult::Data(vec) = sdo_transaction.result else {
        return Err(InitialisationError::ParametrisationCommunicationFailure(
            identifier.clone(),
        ));
    };
    let device_name = std::str::from_utf8(&vec).map_err(|_| {
        InitialisationError::ParametrisationCommunicationFailure(identifier.clone())
    })?;

    if device_name != identifier.device_name {
        return Err(InitialisationError::ParametrisationWrongDeviceName(
            device_name.to_owned(),
            identifier.clone(),
        ));
    }

    trace!("Device Type, Motor Type & Device Name are as expected");
    Ok(())
}

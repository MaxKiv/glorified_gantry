use futures::stream::{Stream, StreamExt};
use gantry_axis::{
    axis::setpoint::{PositionSetpoint, TorqueSetpoint, VelocitySetpoint},
    command::GantryCommand,
};
use gantry_cia402::driver::oms::setpoint::Setpoint;
use r2r::geometry_msgs::msg::Vector3;
use tokio::sync::mpsc;
use tracing::*;
use uom::si::{
    f64::{Length, Torque, Velocity},
    length::millimeter,
    torque::newton_meter,
    velocity::meter_per_second,
};

const PROFILE_VELOCITY_MS: f64 = 0.001;

pub async fn bridge_gantry_setpoints(
    tx: mpsc::Sender<GantryCommand>,
    mut pos_sub: impl Stream<Item = Vector3> + Unpin,
    mut vel_sub: impl Stream<Item = Vector3> + Unpin,
    mut torque_sub: impl Stream<Item = Vector3> + Unpin,
) -> anyhow::Result<()> {
    let velocity = Velocity::new::<meter_per_second>(PROFILE_VELOCITY_MS);

    tokio::join! {
        bridge_pos_setpoints(tx.clone(), pos_sub),
        bridge_vel_setpoints(tx.clone(), vel_sub),
        bridge_torque_setpoints(tx.clone(), torque_sub),
    }
}

async fn bridge_pos_setpoints(
    tx: mpsc::Sender<GantryCommand>,
    mut pos_sub: impl Stream<Item = Vector3> + Unpin,
) -> anyhow::Result<()> {
    while let Some(msg) = pos_sub.next().await {
        info!(
            "Received pos setpoint: x={:.3}, y={:.3}, z={:.3}",
            msg.x, msg.y, msg.z
        );

        let cmd = GantryCommand::Setpoint {
            x: Some(PositionSetpoint {
                target: Length::new::<millimeter>(msg.x),
                velocity,
            }),
            y: Some(PositionSetpoint {
                target: Length::new::<millimeter>(msg.y),
                velocity,
            }),
            z: Some(PositionSetpoint {
                target: Length::new::<millimeter>(msg.z),
                velocity,
            }),
        };

        if let Err(e) = tx.send(cmd).await {
            warn!("Failed to send setpoint: {e:?}");
        }
    }

    Ok(())
}

async fn bridge_vel_setpoints(
    tx: mpsc::Sender<GantryCommand>,
    mut vel_sub: impl Stream<Item = Vector3> + Unpin,
) -> anyhow::Result<()> {
    while let Some(msg) = vel_sub.next().await {
        info!(
            "Received vel setpoint: x={:.3}, y={:.3}, z={:.3}",
            msg.x, msg.y, msg.z
        );

        let cmd = GantryCommand::Setpoint {
            x: Some(VelocitySetpoint {
                target: Velocity::new::<meter_per_second>(msg.x),
            }),
            y: Some(VelocitySetpoint {
                target: Velocity::new::<meter_per_second>(msg.y),
            }),
            z: Some(VelocitySetpoint {
                target: Velocity::new::<meter_per_second>(msg.z),
            }),
        };

        if let Err(e) = tx.send(cmd).await {
            warn!("Failed to send setpoint: {e:?}");
        }
    }

    Ok(())
}

async fn bridge_torque_setpoints(
    tx: mpsc::Sender<GantryCommand>,
    mut torque_sub: impl Stream<Item = Vector3> + Unpin,
) -> anyhow::Result<()> {
    while let Some(msg) = torque_sub.next().await {
        info!(
            "Received torque setpoint: x={:.3}Nm, y={:.3}Nm, z={:.3}Nm",
            msg.x, msg.y, msg.z
        );

        let cmd = GantryCommand::Setpoint {
            x: Some(TorqueSetpoint {
                target: Torque::new::<newton_meter>(msg.x),
            }),
            y: Some(TorqueSetpoint {
                target: Torque::new::<newton_meter>(msg.y),
            }),
            z: Some(TorqueSetpoint {
                target: Torque::new::<newton_meter>(msg.z),
            }),
        };

        if let Err(e) = tx.send(cmd).await {
            warn!("Failed to send setpoint: {e:?}");
        }
    }

    Ok(())
}

use tracing::*;
use uom::si::{
    f64::{Length, Torque, Velocity},
    length::millimeter,
    torque::newton_meter,
    velocity::meter_per_second,
};

type DevicePosition = i32;
type DeviceVelocity = i32;
type DeviceAbsVelocity = u32;
type DeviceTorque = i16;

#[derive(Debug, Clone)]
/// Scaling factors to convert SI units into units used by the motors
pub struct DeviceScaling {
    pub pos_to_ticks: f64,  // counts per millimeter
    pub vel_to_ticks: f64,  // counts/s per m/s
    pub torque_to_raw: f64, // device units (0.1% steps of rated torque) per Nm
}

impl DeviceScaling {
    pub const fn test_setup() -> Self {
        const COUNTS_PER_REV: f64 = 50.0; // Magic guess - I think the test setup has been configured with a feed rate so it takes mm pos as input units
        const LEAD_MM_PER_REV: f64 = 5.0; // Typical, seems right
        const RATED_TORQUE_NM: f64 = 3.1; // From nanotec motor catalog model PD4C6018
        const DEVICE_TORQUE_UNITS_PER_FULL_RATED_TORQUE: f64 = 1000.0; // Nanotec uses 0.1% of rated torque as "torque unit"

        let pos_to_ticks = COUNTS_PER_REV / LEAD_MM_PER_REV; // ticks per mm
        let vel_to_ticks = pos_to_ticks * 1000.0; // ticks/s per (m/s)
        let torque_to_raw = DEVICE_TORQUE_UNITS_PER_FULL_RATED_TORQUE / RATED_TORQUE_NM; // raw units per Nm

        Self {
            pos_to_ticks,
            vel_to_ticks,
            torque_to_raw,
        }
    }

    pub const fn default_setup() -> Self {
        // const COUNTS_PER_REV: f64 = 3600.0; // Default configuration in Cia402Driver
        const COUNTS_PER_REV: f64 = 11500.0; // Magic caliper estimation, must be quick :(
        const LEAD_MM_PER_REV: f64 = 5.0; // Typical, seems right
        const RATED_TORQUE_NM: f64 = 3.1; // From nanotec motor catalog model PD4C6018
        const DEVICE_TORQUE_UNITS_PER_FULL_RATED_TORQUE: f64 = 1000.0; // Nanotec uses 0.1% of rated torque as "torque unit"

        let pos_to_ticks = COUNTS_PER_REV / LEAD_MM_PER_REV; // ticks per mm
        let vel_to_ticks = pos_to_ticks * 1000.0; // ticks/s per (m/s)
        let torque_to_raw = DEVICE_TORQUE_UNITS_PER_FULL_RATED_TORQUE / RATED_TORQUE_NM; // raw units per Nm

        Self {
            pos_to_ticks,
            vel_to_ticks,
            torque_to_raw,
        }
    }

    pub fn to_device_pos(&self, pos: Length) -> DevicePosition {
        let mm = pos.get::<millimeter>();
        (mm * self.pos_to_ticks).round() as DevicePosition
    }

    pub fn to_device_abs_vel(&self, vel: Velocity) -> DeviceAbsVelocity {
        let mut mps = vel.get::<meter_per_second>();
        if mps < 0.0 {
            error!("Attempting to set a absolute velocity below zero: {vel:?}");
            mps = mps.min(0f64);
        }

        (mps * self.vel_to_ticks).round() as DeviceAbsVelocity
    }

    pub fn to_device_vel(&self, vel: Velocity) -> DeviceVelocity {
        let mps = vel.get::<meter_per_second>();
        (mps * self.vel_to_ticks).round() as DeviceVelocity
    }

    pub fn to_device_torque(&self, torque: Torque) -> DeviceTorque {
        let nm = torque.get::<newton_meter>();
        (nm * self.torque_to_raw).round() as DeviceTorque
    }

    pub fn from_device_pos(&self, pos: DevicePosition) -> Length {
        Length::new::<millimeter>(pos as f64 / self.pos_to_ticks)
    }

    pub fn from_device_abs_vel(&self, vel: DeviceAbsVelocity) -> Velocity {
        Velocity::new::<meter_per_second>(vel as f64 / self.vel_to_ticks)
    }

    pub fn from_device_vel(&self, vel: DeviceVelocity) -> Velocity {
        Velocity::new::<meter_per_second>(vel as f64 / self.vel_to_ticks)
    }

    pub fn from_device_torque(&self, torque: DeviceTorque) -> Torque {
        Torque::new::<newton_meter>(torque as f64 / self.torque_to_raw)
    }
}

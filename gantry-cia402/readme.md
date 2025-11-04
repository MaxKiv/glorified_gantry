# Gantry CiA 402

Low-level async Rust driver for CiA 402-compliant motor controllers over CANopen using
`oze-canopen` and `tokio`.

Part of the `MagnetManipulation` project software stack. Using this crate one can send motion commands to any `CiA402` compliant motor,
like target positions, velocities or torques.

## Overview

`gantry-cia402` partially implements the CiA 402 (CANopen device profile for drives and motion control) specification for individual motor control. It provides an event-driven, async interface for commanding motors and receiving feedback.

**Input**: `MotorCommand` enum (Enable, Home, MoveAbsolute, SetVelocity, SetTorque, etc.)
**Output**: `MotorEvent` enum (state updates, position/velocity/torque feedback, faults, etc.)

## Features

- **CiA 402 State Machine**: Full implementation with automatic state transitions
- **Operation Modes**:
  - Profile Position (with handshaking)
  - Profile Velocity
  - Profile Torque
  - Homing (multiple methods)
  - Cyclic Synchronous Position/Velocity/Torque (experimental & _unfinished_)
- **Startup Handling**: Automatic motor parametrization and PDO configuration
- **NMT Management**: Network management with automatic recovery
- **Error Handling**: Fault detection and EMCY message reporting
- **Flexible PDO Mapping**: Configurable TPDO/RPDO layouts (OnChange and OnSync)

## Usage Example

```rust
use gantry_cia402::driver::{Cia402DriverBuilder, command::MotorCommand};

// Feedback timeout
pub const TIMEOUT: Duration = Duration::from_secs(60);
const TEST_POSITION: i32 = -50;
const TEST_SPEED: u32 = 100;

// Initialize CANopen interface
let (canopen, _) = oze_canopen::canopen::start("can0", Some(1_000_000));

// Build a Sync Master, responsible for period SYNC messages
let sync_master = SyncMaster::init(canopen.clone());
let sync_rx = sync_master.get_sync_receiver();

// Build the cia402 driver
let driver = Cia402DriverBuilder::new(node_id)
    .with_canopen(canopen)
    .with_default_pdo_mappings()
    .with_parameters(params)
    .with_sync_receiver(sync_rx)
    .build()
    .await?;

// Send commands - Progress cia402 state machine to Operation Enabled
info!("Sending Command Enable");
driver.cmd_tx.send(MotorCommand::Enable)?;
info!("Wait for Cia402State::OperationEnabled");
wait_for_event(
    drive.event_rx.resubscribe(),
    MotorEvent::Cia402StateUpdate(Cia402State::OperationEnabled),
    TIMEOUT,
)
.await?;

// Send commands - Home this drive using the home method configured in [`params`]
info!("Sending Home command");
drive
    .cmd_tx
    .send(MotorCommand::Home)
    .map_err(DriveError::CommandError)?;

info!("Wait for Homing completed event");
wait_for_event(
    drive.event_rx.resubscribe(),
    MotorEvent::HomingFeedback {
        at_home: true,
        homing_completed: true,
        homing_error: false,
    },
    TIMEOUT,
)
.await?;

info!("Requesting positive absolute position movement to {TEST_POSITION}");
drive
    .cmd_tx
    .send(MotorCommand::MoveAbsolute {
        target: TEST_POSITION,
        profile_velocity: TEST_SPEED,
    })
    .map_err(DriveError::CommandError)?;

info!("Wait for setpoint acknowledged event (Profile Position Handshake)");
wait_for_setpoint_acknowledge(drive.event_rx.resubscribe(), TIMEOUT).await?;

info!("Wait for target reached event");
wait_for_target_reached(drive.event_rx.resubscribe(), TIMEOUT).await?;

// Receive events
while let Ok(event) = driver.event_rx.recv().await {
    match event {
        MotorEvent::PositionFeedback { actual_position } => { /* ... */ }
        MotorEvent::Fault { code, description } => { /* ... */ }
        _ => {}
    }
}

```

# Configuration

Motors are configured via `SdoAction` slices. See `src/driver/startup/params/` for examples:

- `default.rs`: Conservative parameters for production
- `mod.rs`: `TEST_PARAMS` for lab/development setups

PDO mappings are defined in `src/comms/pdo/mapping/`:

- `default.rs`: Verbose OnChange mappings for general use
- `minimal.rs`: Compact OnSync mappings for high-frequency updates
- `cyclic_synchronous.rs`: Specialized mappings for CSP/CSV/CST modes

## Architecture

The driver spawns multiple Tokio tasks that communicate via channels:

```cpp
MotorCommand → [Command Handler]
                    ↓
┌───────────────────┴──────────┐
↓                              ↓
[CiA402 Orchestrator] ← → [State Machine]
↓                             ↓
[Update Publisher] ← → [Setpoint Manager] ← [SYNC Master]
↓                           ↓
[PDO Task] ← ─ ─ ─ ─ → [Feedback Task]
↓                         ↓
CAN Bus MotorEvent → [Subscribers]
```

<img src="../data/cia402_diagram.png" alt="Cia402 Architecture" width="800"/>

# Data Flow

Command Path: MotorCommand → Orchestrator determines state transitions → State Machine validates → Update Publisher centrally combines device updates → PDO Task sends these onto the bus
Feedback Path: CAN Bus → Feedback Task parses frames → MotorEvent published to subscribers
State Coordination: Feedback Task extracts StatusWord → State Machine updates → Orchestrator recalculates path → Update Publisher adjusts ControlWord

The Setpoint Manager handles operating mode specific (OMS) logic (e.g., Profile Position handshaking) and coordinates with the PDO Task for transmission timing (OnChange vs OnSync).

# Performance

Note: these are _preliminary_ and _hardware-dependend_ results included to give the user a general idea.
More profiling is required to provide a robust performance estimate.

Update Rate: Supports at least 100+ Hz single motor command rates
Latency: Typical round-trip <10ms at 1 Mbit/s CAN

See the chapter on CyclicSynchronous SYNC Timing below for more info.

# Design choices

## Async Executor

I've chosen to bake in the `tokio` async executor instead of taking the effort
of being executor agnostic because I'm depending on the `oze-canopen` crate to
manage the CANopen protocol layer and this crate already uses `tokio`
internally.

## CyclicSynchronous SYNC timing

At **1 Mbit/s**, each bit = **1 µs**.
So 1 ms = **1000 bits = 125 bytes total bus capacity** per millisecond.

However, **each CAN frame adds overhead**, so usable payload bandwidth is much lower.

### CAN timing basics

A **Classical CAN frame** (not CAN-FD) has:

| Component              | Bits                      |
| ---------------------- | ------------------------- |
| SOF                    | 1                         |
| Arbitration field (ID) | 11                        |
| Control field          | 6                         |
| Data field             | 0–64                      |
| CRC + ACK + EOF + IFS  | ~50                       |
| **Total (approx)**     | 111 + (8 × payload_bytes) |

That means:

| Payload | Bits      | Bytes on wire |
| ------- | --------- | ------------- |
| 0 bytes | ~111 bits | 14 bytes      |
| 4 bytes | ~143 bits | 18 bytes      |
| 8 bytes | ~175 bits | 22 bytes      |

A good rule of thumb:

> One 8-byte CAN frame ≈ **130–150 µs** total bus time (including inter-frame space).

### Picking SYNC period

| Frame type      | Frames per cycle | Per frame (µs)  | Total (µs) |
| --------------- | ---------------- | --------------- | ---------- |
| SYNC            | 1                | ~130            | 130        |
| RPDO (4 drives) | 4                | ~130            | 520        |
| TPDO (4 drives) | 4                | ~130            | 520        |
| **Total**       | 9                | 130–150 µs each | ≈ 1170 µs  |

Sync period = `130+260*N` microseconds, where `N = number of drives`.

**Result:**
➡️ ~1.17 ms total bus time per cycle for 4 drives.

Even assuming a minimal PDO mapping, 4 fully active drives at 1 kHz **saturate** a 1 Mbit/s bus.

So for 4 drives @ 1 Mbit/s, **2 ms (500 Hz)** SYNC is the sweet spot for general
purpose OS. Still fast but enough margin to ensure stable operation under reasonable
conditions.

### Rule of thumb summary

| Param                | Value            | Meaning                   |
| -------------------- | ---------------- | ------------------------- |
| Bus bitrate          | 1 Mbit/s         | 1 µs per bit              |
| One 8-byte CAN frame | ~130 µs on wire  | ~22 bytes total           |
| Typical SYNC         | 0 B              | ~110 µs                   |
| Usable frames per ms | ~7–8 max         | (~80% bus load safe)      |
| So at 1 kHz:         | ≤ 3–4 drives max | with 1 RPDO + 1 TPDO each |

At 2 ms SYNC, you can comfortably handle up to 8–10 drives.

# Testing

Integration tests require a CAN-adapter connected to CiA 402 compatable hardware:

## Basic CiA 402 state transitions

cargo test -p gantry-cia402 --test cia

## Basic startup procedure

cargo test -p gantry-cia402 --test statup

## Homing procedure

cargo test -p gantry-cia402 --test home

## Profile Position mode movement

cargo test -p gantry-cia402 --test pos

## Profile Torque mode setpoints

cargo test -p gantry-cia402 --test torque

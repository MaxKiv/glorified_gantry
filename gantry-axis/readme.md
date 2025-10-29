# gantry-axis

Multi-axis coordination layer for gantry systems with up to 6 motors across 3 axes.
Part of the `MagnetManipulation` project software stack.

## Overview

`gantry-axis` builds on `gantry-cia402` to orchestrate multiple motors in a gantry configuration. It handles axis initialization, synchronized homing, unit conversions, and coordinated motion commands.

**Input**: `GantryCommand` (Setpoint with X/Y/Z targets, Home)
**Output**: `GantryEvent` (axis position/velocity/torque in SI units, diagnostics, state)

## Features

- **Multi-Motor Axes**: Up to 6 motors total (master + optional slave per axis)
- **SI Unit Interface**: Length (mm), Velocity (m/s), Torque (Nm)
- **Device Scaling**: Configurable translation between SI and motor encoder units
- **Synchronized Operations**: Coordinated homing and setpoint execution
- **SYNC Master**: Built-in SYNC frame generation for cyclic synchronous modes
- **Event Aggregation**: Combines motor events from all axes into unified gantry events

## Usage Example

```rust
use gantry_axis::{gantry::Gantry, command::GantryCommand, axis::setpoint::*};
use uom::si::{f64::Length, length::millimeter};

pub const TEST_X_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::X,
    master: 1,
    slave: None,
    params: TEST_PARAMS,
    scaling: DeviceScaling::test_setup(),
});

pub const TEST_Y_CONFIG: Option<AxisConfig> = None;

pub const TEST_Z_CONFIG: Option<AxisConfig> = Some(AxisConfig {
    axis: Axis::Z,
    master: 3,
    slave: None,
    params: TEST_PARAMS,
    scaling: DeviceScaling::test_setup(),
});

let (canopen, _) = oze_canopen::canopen::start("can0", Some(1_000_000));

let gantry = Gantry::start(
    canopen,
    TEST_X_CONFIG,
    TEST_Y_CONFIG,
    TEST_Z_CONFIG,
    DeviceScaling::test_setup(),
).await?;

// Home all axes
gantry.send_command(GantryCommand::Home).await?;

// Move to position
gantry.send_command(GantryCommand::Setpoint {
    x: Some(AxisSetpoint::AbsolutePosition(PositionSetpoint {
        target: Length::new::<millimeter>(100.0),
        velocity: Velocity::new::<meter_per_second>(0.01),
    })),
    y: None,  // Don't move Y
    z: Some(AxisSetpoint::AbsolutePosition(/* ... */)),
}).await?;

// Receive events
while let Ok(event) = gantry.get_event_rx().recv().await {
    match event {
        GantryEvent::Position { axis, value } => {
            println!("{:?} axis at {:.2} mm", axis, value);
        }
        _ => {}
    }
}
```

## Device Scaling

The `DeviceScaling` struct handles unit conversion:

```rust
pub struct DeviceScaling {
    pub pos_to_ticks: f64,  // counts per mm
    pub vel_to_ticks: f64,  // counts/s per m/s
    pub torque_to_raw: f64, // device units per Nm
}
```

Two presets are provided:

`default_setup()`: Standard 3600 counts/rev configuration for use with the
demostrator setup.
`test_setup()`: Lab configuration with 50 counts/rev for use with the
development setup.

## Testing

Hardware-in-the-loop tests available:

```bash
# Home and move X+Z axes
cargo test -p gantry-axis --test home

# Position mode coordinated motion
cargo test -p gantry-axis --test pos

# Torque mode control
cargo test -p gantry-axis --test torque
```

## Architecture

```
GantryCommand → [Command Handler]
                       ↓
              [Setpoint Translator]
                       ↓
         ┌─────────────┼─────────────┐
         ↓             ↓             ↓
   [X Axis]      [Y Axis]      [Z Axis]
   Motors 1,2    Motors 3,4    Motors 5,6
         ↓             ↓             ↓
   MotorEvent    MotorEvent    MotorEvent
         └─────────────┼─────────────┘
                       ↓
              [Feedback Handler]
                       ↓
              [Event Translator]
                       ↓
                 GantryEvent
```

## Data Flow

### Command Flow:

GantryCommand received by Command Handler
Setpoint Translator converts SI units → device units using DeviceScaling
Command Handler distributes MotorCommands to all motors on each axis
Each motor's Cia402Driver processes independently

### Feedback Flow:

Each motor emits MotorEvents
Axis Event Receivers collect events from master + slave (if present)
Feedback Handler aggregates events by axis
Event Translator converts device units → SI units and wraps as GantryEvent
Subscribers receive unified gantry state

### Synchronization:

SYNC Master broadcasts periodic sync signals
All motors on all axes receive sync via shared channel
Setpoint Manager in each motor sends updates on sync edges

# Gantry Control System

A Rust-based software driver stack for controlling CiA 402-compliant motors in gantry configurations over CANopen from x86 Linux environments. This workspace implements a complete event-driven control system using Tokio's multithreaded async runtime.

## Workspace Structure

This repository contains 6 crates organized as follows:

_Core Libraries_

gantry-cia402 - Low-level CiA 402 motor driver implementation
gantry-axis - Multi-axis gantry coordination layer
gantry-ros2 - ROS2 bridge for integration with robotics systems
gantry-demo - Shared testing utilities and example configurations

_Applications_

gantry-sniffer - CANopen bus monitoring and debugging tool
gantry-gui - Desktop GUI for manual gantry control

_System Requirements_

Platform: x86_64 Linux
Rust: Stable toolchain
CAN Interface: SocketCAN-compatible hardware
Motors: 1+ Cia402-Compatible Motor Drivers

# Quick start

## Set up CAN interface (can0 & 1 Mbit/s)

Use the Just command runner (`just --help` or see justfile):
`just setup-can`

Or manual setup:

```
    sudo ip link set can0 up type can bitrate 1000000
    sudo ip link set can0 txqueuelen 1000
    sudo ip link set up can0

```

## Run the CANopen sniffer to view CANOpen traffic

`just snif` or `cargo run -p gantry-sniffer`

## Run integration tests (requires connected hardware)

`cargo test -p gantry-cia402 --test basic`

# Architecture Overview

The system is built on the `tokio` async runtime and follows a layered architecture:

```
┌─────────────────────────────────────────┐
│ Applications (gantry-gui, gantry-ros2)  │
├─────────────────────────────────────────┤
│ gantry-axis (Multi-axis coordination)   │
├─────────────────────────────────────────┤
│ gantry-cia402 (Single motor control)    │
├─────────────────────────────────────────┤
│ oze-canopen (CANopen protocol)          │
├─────────────────────────────────────────┤
│ SocketCAN (Linux kernel)                │
└─────────────────────────────────────────┘
```

Each layer communicates via async channels using Tokio's broadcast/mpsc
synchronisation primitives.

## Testing

The workspace includes basic tests:

- **Unit tests**: Core logic and state machines
- **Integration tests**: Hardware-in-the-loop tests requiring connected motors
- **Example tests**: Real-world usage scenarios in `gantry-demo/tests/`

Run tests with: `cargo test` (software tests) or `cargo test --test <name>`
(hardware/integration tests).

# FIX

- Cia402Driver: cia402 CW flags are bugged. Sending MotorCmd::Enable + Setpoint
  to motors already in OperationEnabled has correct orchestrator semantics (drive is already
  enabled -> skip) BUT forgets to update the cia402 cw flags causing the setpoint
  RPDO to request cia402 SwitchOnDisabled :(

`2025-10-24T12:28:28.798914Z SNIFF NODE 1  RPDO1 [10, 0, 6]
=> ControlWord(OMS_1) - Homing`

=> Reproduce: reset drives + perform cmd::enable + random setpoint once, see correct operation. Run the same thing again without resetting drives, see bug.
=> Attempted to fix this bug by adding an update from cia402 SM -> pdo, but this
doesnt work fully

- Finish Implementation of Cyclic Synchronous Modes -> just feedback handler is left
  => I fear this isn't very useful on a non-realtime OS.

- Consecutive movement is broken using `PositionModeFlagsCW::DECELERATE_AFTER_REACHING`

- Profile Torque mode has trouble hitting torque target precisely, sometimes this takes
  ages to converge. -> Listen for torqueFeedback { actual_torque: i32 } instead
  and define a target reached margin myself.
  => This is now done more generally in
  `gantry_axis::event::util::wait_for_target_reached`

- Cia402State::OperationEnabled -> SwitchOnDisabled seems broken, should use quick stop transition?

# TODO

- !! Add logic to gantry-axis that quick stops all motors in an axis when one of
  them reports a fault OR reboots.

- ! Figure out how to map T/RPDO's, and how to generalise de/serialisation

- Invalidate all PDOs in the device before mapping, currently TPDO4 isnt in CUSTOM_TPDOS, so
  its never changed from the default and will generate warnings
  -> quick fix by adding EMPTY_TPDO4

- Make error handling uniform across the driver

- Fuzz test orchestrator state orchestrator/machine, this can be done in isolation without CAN, easy wins

!!!FULL DOCUMENTATION WILL BE RELEASED SOON!!!

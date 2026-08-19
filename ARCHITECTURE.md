### Architecture direction

The design converged on three layers:

```text
Gantry / Servo
    │
    └── MotorController(s)
          ├── CiA402 state machine
          ├── handshakes / homing
          └── logical setpoints
                    │
                    ▼
              RT CAN Engine
                    │
          ┌─────────┴─────────┐
          │                   │
      CAN RX/TX              SYNC
```

Keep a reusable `Cia402Motor` per physical motor. Gantry/X-axis coordination stays above it and handles master/slave relationships, skew, etc.

Avoid many Tokio tasks for tightly coupled state. Prefer separate components, but use a task boundary only when concurrency/ownership benefits it.

### RT engine

The RT engine is the **sole CAN owner**, always running at 1 kHz, even outside cyclic modes.

Per cycle:

```text
SYNC
→ collect expected TPDOs
→ validate current-cycle feedback
→ snapshot latest setpoints
→ optional X skew correction
→ transmit RPDOs
→ publish coherent feedback
→ use synchronous window for pending SDOs
```

For non-cyclic modes, it can perform one-shot CAN transactions when commanded.

For cyclic modes, it continuously cycles.

### RT event model

Don't make four async-style event sources. In synchronous Linux Rust, use one blocking event loop around file descriptors:

```text
poll/ppoll
 ├── CAN socket       → TPDO/RX processing
 ├── command eventfd  → RT commands
 └── timerfd          → 1 kHz cycle tick
```

The setpoint snapshot is **not an event**. It is shared memory that RT samples at the defined cycle boundary.

A useful RT state machine is roughly:

```text
Idle
  │ ExecuteOnce
  ▼
SingleCycle

Cyclic
  │ SYNC N
  ▼
WaitingForTpdos(N)
  │
  ├── all required TPDOs → execute cycle
  └── deadline            → fault/timeout policy

Reconfiguring
  │ finish safe cycle
  ├── NMT pre-op
  ├── PDO mapping SDOs
  ├── construct new ActivePdoConfig
  ├── NMT operational
  └── resume
```

Where:

```
WaitingForTpdos(n) = true
  │ execute cycle
  ├── snapshot Pdo Setpoints inside double buffer
  ├── if (x-axis skew > large) -> ESTOP
  ├── if (torque mode) Calculate x axis skew compensation
  ├── Transmit PDO
  ├── Notify tokio of [`CycleFeedback`]
  └── while (time_expired < cycle time (1ms)): Write pending SDO's

SingleCycle
  │ execute single cycle
  ├── SYNC
WaitingForTpdos(N)
  │
  ├── all required TPDOs → execute cycle
  └── deadline            → fault/timeout policy
SingleCycle
  ├── snapshot Pdo Setpoints inside double buffer
  ├── if (x-axis skew > large) -> ESTOP
  ├── if (torque mode) Calculate x axis skew compensation
  ├── Transmit PDO
  ├── Notify tokio of [`CycleFeedback`]
  └── while (time_expired < cycle time (1ms)): Write pending SDO's
```

Don't stop/restart parts of the RT thread. Tokio sends a `Reconfigure` command; the RT thread performs the transition itself at a safe cycle boundary.

### TPDO cycle association

Don't expect the CAN frame to contain “cycle N”.

RT owns a monotonic cycle counter:

```text
SYNC → cycle = N → enter WaitingForTpdos(N)
```

Each expected TPDO received after that is associated with N.

Track:

```rust
received_cycle
timestamp
valid
```

If a TPDO is missing, don't silently treat cycle N-1 as current. Retain last-known data for diagnostics, but mark current feedback invalid and apply an explicit timeout/fault policy.

### PDO mapping

Keep active PDO configuration RT-owned and immutable during a running cycle:

```rust
struct ActivePdoConfig {
    tx_mapping: ...,
    rx_mapping: ...,
    expected_tpdos: ...,
}
```

On mode/PDO remapping:

```text
Tokio → RtCommand::Reconfigure
RT:
    finish current cycle
    stop cyclic processing
    NMT pre-op
    configure mappings
    build new decoder/encoder config
    install atomically
    NMT operational
    resume
```

This avoids Tokio mutating a mapping while RT is decoding with it.

### Tokio ↔ RT data

Use different mechanisms for different semantics.

**Commands:** queue + eventfd notification.

```text
Tokio
 ├── push RtCommand
 └── eventfd_write()
          ↓
        RT poll()
```

Commands include:

```rust
StartCyclic
StopCyclic
Reconfigure(...)
ExecuteOnce
Shutdown
```

**Cyclic setpoints:** latest-value shared snapshot, not a queue.

A single task (for both gantry and single servo case) waits for recv of all
`MotorCycleSetpoint` with current `generation` and collects that into a
`CycleSetpoint` for that generation. Upon collection of new `CycleSetpoint`, the
RT setpoint snapshot (double buffer?) is updated so the RT thread can access it.

```rust
struct MotorCycleSetpoint {
    generation: u64,
    controlword: u16,
    target: i32,
    OR
    target: MotorSetpoint(ProfilePosition(10))
}

struct CycleSetpoint<const N: usize> {
    generation: u64,
    motors: [MotorCycleSetpoint; N],
}
```

Use double buffering / another SPSC snapshot mechanism so RT gets one consistent
set of all motor setpoints per cycle. Avoid mutexes/allocations in the RT path.

**Feedback:** publish one coherent cycle snapshot, e.g.:

```rust
struct CycleFeedback<const N: usize> {
    cycle: u64,
    motors: [MotorFeedback; N],
    timing: CycleTiming,
    errors: RtErrors,
    skew: Option<f64>?
}
```

Tokio can be notified via eventfd, while the actual data remains in a latest-value buffer.

Gantry can parse this into

```rust
struct GantryFeedback {
    x: AxisFeedback,
    y: AxisFeedback,
    z: AxisFeedback,
    cycle: u64,
    timing: CycleTiming,
    errors: GantryErrors,
}
```

Single servo case can parse into a `MotorFeedback` or similar.

### Important correction from earlier discussion

The RT engine should not contain all CiA402 semantics. It should know:

- PDO mappings
- CAN IDs
- decoding/encoding
- SYNC
- timing
- current-cycle feedback
- cyclic transmission

The `MotorController` should know:

- CiA402 state transitions
- setpoint semantics
- PP handshakes
- homing
- mode semantics

So the core boundary is:

> **MotorController decides what should happen; RT/PDO engine decides when/how process data reaches CAN.**

### Linux resources worth learning

Given your embedded RT background, focus on Linux-specific pieces:

- `std::thread`, `Send`/`Sync`, atomics, Acquire/Release
- Linux `SCHED_FIFO`, affinity, `PREEMPT_RT`
- `poll`/`ppoll`/`epoll`
- `timerfd`
- `eventfd`
- SocketCAN raw sockets/filtering
- SPSC/double-buffer shared-memory patterns

Good primary references:

- Linux real-time docs: [https://www.kernel.org/doc/html/latest/core-api/real-time/](https://www.kernel.org/doc/html/latest/core-api/real-time/)
- `timerfd`: [https://man7.org/linux/man-pages/man2/timerfd_create.2.html](https://man7.org/linux/man-pages/man2/timerfd_create.2.html)
- `eventfd`: [https://man7.org/linux/man-pages/man2/eventfd.2.html](https://man7.org/linux/man-pages/man2/eventfd.2.html)
- Tokio `AsyncFd`: [https://docs.rs/tokio/latest/tokio/io/unix/struct.AsyncFd.html](https://docs.rs/tokio/latest/tokio/io/unix/struct.AsyncFd.html)
- Rustonomicon atomics: [https://doc.rust-lang.org/nomicon/atomics.html](https://doc.rust-lang.org/nomicon/atomics.html)
- SocketCAN: [https://www.kernel.org/doc/html/latest/networking/can.html](https://www.kernel.org/doc/html/latest/networking/can.html)

The next concrete implementation milestone is a tiny prototype with **one Tokio producer + one `std::thread` RT loop + shared setpoint snapshot + command queue/eventfd + SocketCAN + 1 kHz `timerfd`**, before integrating the full CiA402 stack.

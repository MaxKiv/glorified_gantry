# Gantry CiA 402

Part of the `MagnetManipulation` project software stack. Implements logic to
drive any `CiA402` compliant motor/driver. Implements the `CiA402` state machine
and transition logic, manages the `Controlword (0x6040)`, `Statusword
(0x6041)` and `OperationalMode (0x6060/0x6061)` Object Dictionary entries.

Using this crate one can send motion commands to any `CiA402` compliant motor,
like target positions, velocities or torques.

Requires a `CANopen` protocol manager as defined by the `CANopenNode` trait.

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

**Result:**
➡️ ~1.17 ms total bus time per cycle.

Even assuming tight bit-packing and minimal inter-frame gaps, 4 fully active drives at 1 kHz **saturate** a 1 Mbit/s bus.

| SYNC period        | Typical use case                 | Comment                     |
| ------------------ | -------------------------------- | --------------------------- |
| **1 ms (1 kHz)**   | High-performance, ≤ 2 drives     | Tight, but possible         |
| **2 ms (500 Hz)**  | Common compromise for 3–5 drives | Still feels smooth          |
| **4 ms (250 Hz)**  | Standard industrial default      | Enough for most linear axes |
| **10 ms (100 Hz)** | Low-cost systems                 | Noticeable coarseness       |

So for 4 drives @ 1 Mbit/s, **2 ms (500 Hz)** SYNC is the sweet spot — safe margin, low jitter, and easy CPU timing.

### Rule of thumb summary

| Param                | Value            | Meaning                   |
| -------------------- | ---------------- | ------------------------- |
| Bus bitrate          | 1 Mbit/s         | 1 µs per bit              |
| One 8-byte CAN frame | ~130 µs on wire  | ~22 bytes total           |
| Typical SYNC         | 0 B              | ~110 µs                   |
| Usable frames per ms | ~7–8 max         | (~80% bus load safe)      |
| So at 1 kHz:         | ≤ 3–4 drives max | with 1 RPDO + 1 TPDO each |

At 2 ms SYNC, you can comfortably handle up to 8–10 drives.

### Left off:

Figure out:

1. how to switch pdo configuration?
2. how to parse PDO on CAN_RX
3. how to map "All motors setpoints" -> current active PDO mapping

4. Should Profile modes RPDO transmission type be "onchange" or "onsync"?
5. Should Cyclic Profile mode use TPDO2?
6. Validate TEST/DEMOSTRATOR HGantryNodeMap
7. Merge HGantryNodeMap -> AxisConfiguration?
8. Move gantry specific stuff to its own crate

## Improvements

### Move Gantry specific stuff to its own crate

Yea that

### Heartbeat / Node guarding protocol

RT thread should parse Heartbeat/node guarding msgs in CAN_RX, and trigger some
fault policy if drive fails.

### Cycle budget

Document expected costs per phase:

```
1kHz cycle budget (1000µs total):
  • timerfd wake       5µs
  • poll() dispatch    10µs
  • CAN RX processing  20µs
  • TPDO decode        15µs
  • setpoint snapshot  5µs
  • RPDO encode        15µs
  • CAN TX dispatch    20µs
  • feedback publish   10µs
  • headroom           500µs  ← critical!
```

Measure these at runtime, output these as part of feedback. Perhaps Error on
timeouts

### eventfd coalescing

Command queue + eventfd: watch the coalescing

eventfd in default (non-EFD_SEMAPHORE) mode coalesces notifications — one wakeup does not mean one command. RT must drain the queue to empty on every wakeup, not assume 1:1. Minor but easy to get subtly wrong.

### Testability

Nothing about abstracting the CAN transport for testing. Recommend a trait around the socket so you can run the RT loop against vcan (or an in-memory fake bus) in CI without hardware — useful for exercising the Reconfiguring/WaitingForTpdos/timeout paths deterministically.

### Left off:

Started on motor-controller, supposed to be a cia402driver impl with less tasks.

- impl motor-controller
- finish gantry-controller
- write rt_engine

## Improvements

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

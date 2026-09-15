### Left off:

/home/max/git/saxion/glorified_gantry/motor-controller/src/canopen/pdo/mapping.rs:35

## To Figure out:

Now:

1. Create some sort of CanOpen structure with 2 sides, 1 tx side:
   - accepts CanOpen concepts (nmt send, sdo cmd enqueue, etc)
   - holds these in some internal queue?
   - can poll rx side for milestones?
     and another rx side that:
   - drains internal queue, sends out canframes
   - tracks milestones, like sdo download confirms?

This allows us to schedule the canopen traffic at some deterministic point in
the sync cycle, and allows us to yield and wait for sdo/nmt commands to confirm.

Later:

2. Should Profile modes RPDO transmission type be "onchange" or "onsync"?
3. Validate TEST/DEMOSTRATOR HGantryNodeMap

## Improvements

### Add feedback to NMT system

Currently I assume the drives always switch into the requested state

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

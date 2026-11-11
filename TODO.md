## To Figure out:

/home/max/git/saxion/glorified_gantry/motor-controller/src/motor/script/parametrisation.rs:39

First:

- [ ] Complete Cia402Motor Scripting engine (tick(), poll())
- [ ] figure out how to do Cia402 transitions in Cia402Motor
- [ ] Cia402Motor RPDO scheduling in profile modes?
- [ ] Figure out how & when to write CAN messages in RT loop?
- [ ] Cia402Motor must write can sometimes, pass &mut CanOpen struct everywhere
- [ ] Compile this shit
- [ ] Construct CycleFeedback
- [ ] Start testing, start small -> single drive bringup (parametrisation)

Later:

1. Should Profile modes RPDO transmission type be "onchange" or "onsync"?
1. Validate TEST/DEMOSTRATOR HGantryNodeMap

## Improvements

### Allow for const scripts

avoid rebuilding ESTOP et al at runtime
likely all scripts could be generated at compile or startup time

### Add motor scaling

Currently I assume

````rust
    pub fn update_from_motor_feedback(&mut self, feedback: &MotorFeedback) {
        self.pos = Length::new::<millimeter>(feedback.pos as f64);
        self.vel = Velocity::new::<millimeter_per_second>(feedback.vel as f64);
        self.torque = Torque::new::<newton_meter>(feedback.torque as f64);
    }
    ```

### Add feedback to NMT system

Currently I assume the drives always switch into the requested state

### Heartbeat / Node guarding protocol

RT thread should parse Heartbeat/node guarding msgs in CAN_RX, and trigger some
fault policy if drive fails.

### Cycle budget

Document expected costs per phase:

````

1kHz cycle budget (1000µs total):
• timerfd wake 5µs
• poll() dispatch 10µs
• CAN RX processing 20µs
• TPDO decode 15µs
• setpoint snapshot 5µs
• RPDO encode 15µs
• CAN TX dispatch 20µs
• feedback publish 10µs
• headroom 500µs ← critical!

```

Measure these at runtime, output these as part of feedback. Perhaps Error on
timeouts

### eventfd coalescing

Command queue + eventfd: watch the coalescing

eventfd in default (non-EFD_SEMAPHORE) mode coalesces notifications — one wakeup does not mean one command. RT must drain the queue to empty on every wakeup, not assume 1:1. Minor but easy to get subtly wrong.

### Testability

Nothing about abstracting the CAN transport for testing. Recommend a trait around the socket so you can run the RT loop against vcan (or an in-memory fake bus) in CI without hardware — useful for exercising the Reconfiguring/WaitingForTpdos/timeout paths deterministically.
```

# FIX

- Finish Implementation of Cyclic Synchronous Modes -> just feedback handler is left

- Consecutive movement is broken using `PositionModeFlagsCW::DECELERATE_AFTER_REACHING`

- Profile Torque mode has trouble hitting torque target precisely, sometimes this takes
  ages to converge. -> Listen for torqueFeedback { actual_torque: i32 } instead
  and define a target reached margin myself.

- Cia402State::OperationEnabled -> SwitchOnDisabled seems broken, should use quick stop transition?

# TODO

- ! Figure out how to map T/RPDO's, and how to generalise de/serialisation

- Give every motor a String name, derive it from node_id by default

- Invalidate all PDOs in the device before mapping, currently TPDO4 isnt in CUSTOM_TPDOS, so
  its never changed from the default and will generate warnings

- Make error handling uniform across the driver

- Unit test applicable logic, like bit fiddling/merging

- Fuzz test orchestrator state orchestrator/machine, this can be done in isolation without CAN, easy wins

# Set up physical CAN

```bash
sudo ip link set can0 up type can bitrate 1000000
sudo ip link set can0 txqueuelen 1000
sudo ip link set up can0
```

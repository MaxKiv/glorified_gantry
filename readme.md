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

warning: Git tree '/home/max/git/saxion/ros2_canopen' is dirty
Resolved URL: git+file:///home/max/git/saxion/ros2_canopen
Description: ROS 2 development environment using nix
Path: /nix/store/plgybjyylh40k9vfxgl1yaw8sh3ysyim-source
Revision: f6be67da6a71b097d34bf721bddb18d63abf0c63-dirty
Last modified: 2025-07-22 14:57:41
Fingerprint: fc1d811be2fb8d23980e4c9526ba2d0926b5797a887175532f97115faab174a6

warning: Git tree '/home/max/git/saxion/glorified_gantry' is dirty
Resolved URL: git+file:///home/max/git/saxion/glorified_gantry
Description: Nanotec nanolib example CLI build environment
Path: /nix/store/y3sa6638xnw9xla6cdiaqxjip6isyd1k-source
Revision: 48e611108427bc6a152583eb1cc130c620f5109a-dirty
Last modified: 2025-10-24 08:44:00
Fingerprint: 857cc4538a95045113d7b01cb5597968a754efb67a8f4464bf4e698246568f86

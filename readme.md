# TODO

- Give every motor a String name, derive it from node_id by default

- Orchestrate task spawning, at least make startup task run first

- Move setpoint generation functionality into OMS handler. e.g. flags and
  specifics, change the setpoint_cmd_tx to pass MotorCommand?

- ! Merge the update publisher + Pdo struct into 1, when writing PDO's we have
  access to everything (eg. controlword + setpoint & flags), so we should construct the PDO there

- Make Specific RPDO/TPDO mappings required, encode them into the type system
  somehow. Building something that can work with any generic mapping is hard and
  not efficient use of my time. Plus our current R/TPDO mapping is always
  required if you want to run a tight setup

# Set up physical CAN

```bash
sudo ip link set can0 up type can bitrate 1000000
sudo ip link set can0 txqueuelen 1000
sudo ip link set up can0
```

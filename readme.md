# TODO

- Give every motor a String name, derive it from node_id by default

- Orchestrate task spawning, at least make startup task run first

- Move setpoint generation functionality into OMS handler. e.g. flags and
  specifics, change the setpoint_cmd_tx to pass MotorCommand?

- ! Merge the update publisher + Pdo struct into 1, when writing PDO's we have
  access to everything (eg. controlword + setpoint & flags), so we should construct the PDO there

- Parse R/TPDO mapping at configuration time or encode into type system. current
  hardcode setup is brittle

# Set up physical CAN

```bash
sudo ip link set can0 up type can bitrate 1000000
sudo ip link set can0 txqueuelen 1000
sudo ip link set up can0
```

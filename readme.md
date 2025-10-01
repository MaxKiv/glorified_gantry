# TODO

- Give every motor a String name, derive it from node_id by default

- Orchestrate task spawning, at least make startup task run first

- Move setpoint generation functionality into OMS handler. e.g. flags and
  specifics, change the setpoint_cmd_tx to pass MotorCommand?

# Set up physical CAN

```bash
sudo ip link set can0 up type can bitrate 1000000
sudo ip link set can0 txqueuelen 1000
sudo ip link set up can0
```

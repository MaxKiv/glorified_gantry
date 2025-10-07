# TODO

- Give every motor a String name, derive it from node_id by default

- Orchestrate task spawning, at least make startup task run first

- Re-use updated parsing logic in driver::feedback::frame::parse

- Parse R/TPDO mapping at configuration time or encode into type system. current hardcode setup is brittle

- Refactor the feedback task to be more generic

# Set up physical CAN

```bash
sudo ip link set can0 up type can bitrate 1000000
sudo ip link set can0 txqueuelen 1000
sudo ip link set up can0
```

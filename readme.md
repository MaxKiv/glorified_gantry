# TODO

- Give every motor a String name, derive it from node_id by default

- Orchestrate task spawning, at least make startup task run first

- Parse R/TPDO mapping at configuration time or encode into type system. current hardcode setup is brittle

- Invalidate all PDOs before mapping, currently TPDO4 isnt in CUSTOM_TPDOS, so
  its never changed from the default and will generate warnings

- All R/TPDO code makes heavy assumptions on the R/TPDO mapping, which makes it
  hard to change anything. Generalising would be better.

# Set up physical CAN

```bash
sudo ip link set can0 up type can bitrate 1000000
sudo ip link set can0 txqueuelen 1000
sudo ip link set up can0
```

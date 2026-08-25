help:
    @just --list

# Enable CAN interface
setup-can:
    sudo ip link set can0 up type can bitrate 1000000
    sudo ip link set can0 txqueuelen 1000
    sudo ip link set up can0

# Clean project build artifacts
clean:
    cargo clean

# Check if project compiles
check:
    cargo check

# Run a CANopen sniffer
snif:
    cargo run -p gantry-sniffer

# Show the current canbus load
show-can-load:
    canbusload can0@1000000 -rbtscv

spam-can:
    for i in {0..1000}; do cansend can0 123#DEADBEEF; done

# Gantry CiA 402

Part of the `MagnetManipulation` project software stack. Implements logic to
drive any `CiA402` compliant motor/driver. Implements the `CiA402` state machine
and transition logic, manages the `Controlword (0x6040)`, `Statusword
(0x6041)` and `OperationalMode (0x6060/0x6061)` Object Dictionary entries.

Using this crate one can send motion commands to any `CiA402` compliant motor,
like target positions, velocities or torques.

Requires a `CANopen` protocol manager as defined by the `CANopenNode` trait.

# Design choices

I've chosen to bake in the `tokio` async executor instead of taking the effort
of being executor agnostic because I'm depending on the `oze-canopen` crate to
manage the CANopen protocol layer and this crate already uses `tokio`
internally.

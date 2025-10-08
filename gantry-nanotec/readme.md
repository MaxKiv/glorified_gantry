# Gantry NanoTec Canopen

Part of the MagnetManipulation project software stack responsible for handling
NanoTec specific quirks and parametrisation. Builds on the `gantry_cia402` crate
that provides a more abstract `CiA402` state machine & motor management system.

# Design choices

I've chosen to bake in the `tokio` async executor instead of taking the effort
of being executor agnostic because I'm depending on the `oze-canopen` crate to
manage the CANopen protocol layer and this crate already uses `tokio`
internally.

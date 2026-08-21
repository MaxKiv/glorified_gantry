pub mod channel;
pub mod queue;

#[derive(Clone, Copy, Debug)]
pub enum RtCommand {
    Shutdown,
    Reconfigure,
    SingleCycle, //?
    Cyclic,      //?
}

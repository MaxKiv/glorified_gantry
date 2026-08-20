pub mod channel;
pub mod queue;

#[derive(Clone, Copy, Debug)]
pub enum RtCommand {
    Test,
    Shutdown,
    UrMom,
}

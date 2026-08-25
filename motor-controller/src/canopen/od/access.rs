#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Const,
}

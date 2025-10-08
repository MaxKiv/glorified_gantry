#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Const,
}

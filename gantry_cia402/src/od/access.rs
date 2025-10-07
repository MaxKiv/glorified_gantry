#[derive(Debug, Clone, Copy)]
pub enum AccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Const,
}

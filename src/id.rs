use crate::util::IdPath;

pub type SymbolId = (IdModule, IdFile, IdItem);

pub type IdModule = IdPath;
pub type IdFile = u16;

#[derive(Debug, Copy, Clone)]
pub enum IdItem {
    // Variable
    GlobalVar(u16),

    /// Pointing to a function declared in the global scope
    Func(u16),

    /// Points to a Type declared in the global scope
    Type(u16),

    /// Describes a auto-derived Method in terms of referencing the type and specific field OR Enum and specific Variant
    /// Note, enum variants MAY be constant
    Method(u16, u16),
}

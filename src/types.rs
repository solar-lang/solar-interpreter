use crate::id::{IdModule, TypeId};

/// Represents a concrete, static type
/// and the information needed to construct it.
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Type {
    /// Module used for looking up functions associated with this type
    module: IdModule,
    size_in_bytes: u16,
    // Types don't really need fieldnames I guess
    fields: Vec<TypeId>,
}

pub mod buildin;

use crate::id::{IdModule, TypeId};

/// Represents a concrete, static type
/// and the information needed to construct it.
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Type {
    /// Module used for looking up functions associated with this type
    pub info_name: String,
    module: IdModule,
    size_in_bytes: u32,
    field_layout: Vec<(String, u32, TypeId)>,
}

impl Type {
    /// returns the offset, length and TypeId of the given field
    pub fn get_field(&self, name: &str) -> Result<(u32, u32, TypeId), ()> {
        for (index, (n, offset, id)) in self.field_layout.iter().enumerate() {
            if n != name {
                continue;
            }

            let next = if index == self.field_layout.len() {
                self.size_in_bytes
            } else {
                self.field_layout[index].1
            };

            let len = next - offset;

            return Ok((*offset, len, *id));
        }

        Err(())
    }
}

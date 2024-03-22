use crate::id::TypeId;

#[derive(Debug, Clone, Default)]

/// Logical Scope, used for looking up variable names in scope.
/// Since variable names can overshadow each other,
/// the scope respects that when looking up,
/// and only returns the most recent variable.
/// optimized for small number of entries.
/// Optimized for pushing and popping.
pub struct Scope {
    values: Vec<(String, TypeId, u16)>,
    /// counts how many times has been pushed into this scope,
    /// and assiciates a value with it.
    /// This is done to differentiate values added to this scope with certainty.
    /// Even, if they have the same name, they will have differing values.
    counter: u16,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            values: vec![],
            counter: 0,
        }
    }

    pub fn get(&self, name: &str) -> Option<(TypeId, u16)> {
        self.values
            .iter()
            .rfind(|(n, _, _)| n == name)
            .map(|(_, ty, index)| (*ty, *index))
    }

    pub fn push(&mut self, name: impl Into<String>, ty: TypeId) -> u16 {
        let index = self.counter;
        self.counter += 1;
        self.values.push((name.into(), ty, index));
        index
    }

    /// Pops the most recent value out of the scope.
    /// Popping of an empty scope is considered a programming error
    /// and results in a panic.
    pub fn pop(&mut self) -> (TypeId, u16) {
        let (_, a, b) = self.values.pop().expect("find value in local scope");
        (a, b)
    }
}

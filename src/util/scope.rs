#[derive(Debug, Clone, Default)]
/// Logical Scope, optimized for small number of entries.
/// Made so pushing and popping works fine.
pub struct Scope<'a> {
    values: Vec<(String, Value<'a>)>,
}

impl<'a> Scope<'a> {
    pub fn new() -> Self {
        Scope::default()
    }

    pub fn get(&self, name: &str) -> Option<&Value<'a>> {
        self.values.iter().rfind(|(n, _)| n == name).map(|(_, v)| v)
    }

    pub fn push(&mut self, name: &str, value: Value<'a>) {
        self.values.push((name.to_string(), value));
    }

    /// Pops the most recent value out of the scope.
    /// Popping of an empty scope is considered a programming error
    /// and results in a panic.
    pub fn pop(&mut self) -> Value<'a> {
        self.values.pop().expect("find value in local scope").1
    }
}

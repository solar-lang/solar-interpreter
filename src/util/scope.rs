#[derive(Debug, Clone, Default)]
/// Logical Scope, optimized for small number of entries.
/// Optimized for pushing and popping.
pub struct Scope<T> {
    values: Vec<(String, T)>,
}

impl<T> Scope<T> {
    pub fn new() -> Self {
        Scope::default()
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        self.values.iter().rfind(|(n, _)| n == name).map(|(_, v)| v)
    }

    pub fn push(&mut self, name: impl Into<String>, value: T) {
        self.values.push((name.into(), value));
    }

    /// Pops the most recent value out of the scope.
    /// Popping of an empty scope is considered a programming error
    /// and results in a panic.
    pub fn pop(&mut self) -> T {
        self.values.pop().expect("find value in local scope").1
    }
}

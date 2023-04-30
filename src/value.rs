use std::rc::Rc;

pub type GcString = Rc<String>;

/// Represents a Dynamically Typed Value
#[derive(Debug, Clone)]
pub enum Value {
    Void,
    Bool(bool),
    Int64(i64),
    String(GcString),
}

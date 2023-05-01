use std::rc::Rc;

pub type GcString = Rc<String>;

/// Represents a Dynamically Typed Value
#[derive(Debug, Clone)]
pub enum Value {
    Void,
    Bool(bool),
    Int(Int),
    Float(f64),
    String(GcString),
}

#[derive(Debug, Clone, Copy)]
pub enum Int {
    Int64(i64),
    Int32(i32),
    Int16(i16),
    Int8(i8),
    Uint64(u64),
    Uint32(u32),
    Uint16(u16),
    Uint8(u8),
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(Rc::new(value))
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Void
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

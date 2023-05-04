use solar_parser::ast;
use std::{
    fmt::{self, Debug},
    rc::Rc,
};

pub type GcString = Rc<String>;

type GenericFn<'a> = Rc<dyn Fn(&[Value<'a>]) -> Value<'a>>;

/// Represents a Dynamically Typed Value
#[derive(Clone)]
pub enum Value<'a> {
    Void,
    Bool(bool),
    Int(Int),
    Float(f64),
    String(GcString),
    FnBuildin(GenericFn<'a>),
    AstFunction(&'a ast::Function<'a>),
}

impl Value<'_> {
    pub fn type_as_str(&self) -> &'static str {
        match self {
            Value::Void => "Void",
            Value::Bool(_) => "Bool",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::String(_) => "String",
            Value::FnBuildin(_) => "Fn",
            Value::AstFunction(_) => "Fn",
        }
    }
}

impl Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ty = self.type_as_str();

        write!(f, "({ty}) {self}")
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Void => write!(f, "void"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(i) => write!(f, "{i}"),
            Value::String(i) => write!(f, "{i}"),
            Value::FnBuildin(_) => Ok(()),
            Value::AstFunction(_) => Ok(()),
        }
    }
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

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Int::*;
        match self {
            Int64(v) => write!(f, "{v}"),
            Int32(v) => write!(f, "{v}"),
            Int16(v) => write!(f, "{v}"),
            Int8(v) => write!(f, "{v}"),
            Uint64(v) => write!(f, "{v}"),
            Uint32(v) => write!(f, "{v}"),
            Uint16(v) => write!(f, "{v}"),
            Uint8(v) => write!(f, "{v}"),
        }
    }
}

impl<'a> From<&'a ast::Function<'a>> for Value<'a> {
    fn from(value: &'a ast::Function<'a>) -> Self {
        Value::AstFunction(value)
    }
}

impl From<String> for Value<'_> {
    fn from(value: String) -> Self {
        Value::String(Rc::new(value))
    }
}

impl From<()> for Value<'_> {
    fn from(_: ()) -> Self {
        Value::Void
    }
}

impl From<bool> for Value<'_> {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl<'a, T: 'static + Fn(&[Value<'a>]) -> Value<'a>> From<T> for Value<'a> {
    fn from(value: T) -> Self {
        Value::FnBuildin(Rc::new(value))
    }
}

mod function_context;
mod interpreter;
mod context;

pub use context::*;

pub use function_context::*;
use hotel::HotelMap;
use interpreter::InterpreterContext;
use solar_parser::ast;

use crate::{
    project::{FindError, },
    value::Value,
};
use thiserror::Error;


#[derive(Debug, Error)]
pub enum EvalError {
    IntConversion(#[from] std::num::ParseIntError),
    FindError(#[from] FindError),
    WrongBuildin { found: String },
    TypeError { got: String, wanted: String },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IntConversion(e) => e.fmt(f),
            Self::FindError(e) => e.fmt(f),
            Self::WrongBuildin { found } => {
                write!(f, "only buildin methods are allowed to start with buildin_ or Buildin_.\n Found {found}.")
            }

            Self::TypeError { got, wanted } => {
                write!(f, "Wrong type supplied. Expected {wanted}, got {got}")
            }
        }
    }
}

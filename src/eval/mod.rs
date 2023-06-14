mod context;
mod interpreter;

pub use context::*;

use crate::project::FindError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvalError {
    IntConversion(#[from] std::num::ParseIntError),
    FindError(#[from] FindError),
    WrongBuildin {
        found: String,
    },
    TypeError {
        got: String,
        wanted: String,
    },
    /// Variables musn't be called
    CallingVariable {
        identifer: String,
        file: String,
    },
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

            Self::CallingVariable { identifer, file } => {
                write!(f, "tried to call variable {identifer} in {file}. Don't supply arguments to variables, it will be interpreted as a function call.")
            }
        }
    }
}

mod file_context;
mod interpreter;

use interpreter::InterpreterContext;

use crate::{
    project::{FindError, GlobalModules, Module, ProjectInfo},
    value::Value,
};
use std::{
    io::{Read, Write},
    sync::Mutex,
};
use thiserror::Error;

pub struct CompilerContext<'a> {
    /// Information about all loaded dependencies and sub-dependencies, flattend.
    pub project_info: ProjectInfo,
    /// contains all ASTs across all modules and (sub-)dependencies
    pub module_info: GlobalModules<'a>,

    pub interpreter_ctx: Mutex<InterpreterContext>,
}

impl<'a> CompilerContext<'a> {
    /// Creates a new Compiler Context with stdin and stdout
    /// propagated
    pub fn with_default_io(project_info: ProjectInfo, module_info: GlobalModules<'a>) -> Self {
        CompilerContext {
            project_info,
            module_info,
            interpreter_ctx: Mutex::new(InterpreterContext::default()),
        }
    }

    /// Resolve module based on idpath
    fn resolve_module(&self, idpath: &[String]) -> Option<&Module<'a>> {
        self.module_info.get(idpath)
    }

    fn buildin_str_concat(&self, args: &[Value]) -> Result<Value, EvalError> {
        let mut s = String::new();

        for arg in args {
            match arg {
                Value::String(arg) => s.push_str(arg),
                _ => {
                    return Err(EvalError::TypeError {
                        got: arg.type_as_str().to_string(),
                        wanted: "String".to_string(),
                    })
                }
            }
        }

        Ok(s.into())
    }

    fn buildin_print(&self, args: &[Value]) -> Result<Value, EvalError> {
        // allowed overloadings:
        // [String]
        // []

        let mut out = self.interpreter_ctx.lock().expect("lock interpreter io");
        for arg in args {
            write!(*out, "{arg}").expect("write to interpreter io");
        }
        out.flush().expect("write to interpreter io");

        Ok(Value::Void)
    }

    fn buildin_identity(&'a self, args: &[Value<'a>]) -> Result<Value<'a>, EvalError> {
        // only the identiy overloading is implemented for now.
        if args.len() != 1 {
            panic!("& is only implemented with 1 argument");
        }

        Ok(args[0].clone())
    }

    fn buildin_readline(&self, args: &[Value]) -> Result<Value, EvalError> {
        let mut iio = self.interpreter_ctx.lock().expect("lock interpreter io");

        // allowed overloadings:
        // [String]
        // []
        if !args.is_empty() {
            // Check that no more than 1 argument got supplied
            if args.len() > 1 {
                panic!("Expected 1 argument of type string to buildin_readline");
            }

            // Verify that it is of type string
            let s = if let Value::String(s) = &args[0] {
                s
            } else {
                panic!("Expected argument to buildin_readline to be of type string");
            };

            write!(iio, "{s}").expect("write to interpreter io");
            iio.flush().expect("flush interpreter io");
        }

        let mut s = Vec::new();

        loop {
            // read exactly one character
            let mut buf = [0];
            iio.read_exact(&mut buf).expect("read from input");

            // grab buffer as character
            let b = buf[0];

            if b == b'\n' {
                break;
            }

            s.push(b)
        }

        let s = String::from_utf8(s).expect("parse stdin as a string");
        Ok(s.into())
    }
}

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

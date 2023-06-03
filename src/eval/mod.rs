mod function_context;
mod interpreter;

pub use function_context::*;
use interpreter::InterpreterContext;
use solar_parser::ast;

use crate::{
    project::{FindError, FunctionInfo, GlobalModules, Module, ProjectInfo},
    util,
    value::Value,
};
use std::{
    io::{Read, Write},
    sync::Mutex,
};
use thiserror::Error;

/// Struct that gets created once globally
/// Containing Information about all Modules, ASTs, Projects
pub struct CompilerContext<'a> {
    /// Information about all loaded dependencies and sub-dependencies, flattend.
    pub project_info: &'a ProjectInfo,
    /// contains all ASTs across all modules and (sub-)dependencies
    pub module_info: GlobalModules<'a>,

    /// Contains runtime configurations, like stdin and stdout
    pub interpreter_ctx: Mutex<InterpreterContext>,
}

impl<'a> CompilerContext<'a> {
    /// Creates a new Compiler Context with stdin and stdout
    /// propagated
    pub fn with_default_io(project_info: &'a ProjectInfo, module_info: GlobalModules<'a>) -> Self {
        CompilerContext {
            project_info,
            module_info,
            interpreter_ctx: Mutex::new(InterpreterContext::default()),
        }
    }

    /// Finds the main function of the current target project
    pub fn find_target_main(&'a self) -> Result<FunctionInfo<'a>, FindError> {
        let path = util::target_id();
        let module = self.module_info.get(&path).unwrap();

        let mut candidates = module.find("main")?;
        if candidates.len() != 1 {
            return Err(FindError::TooMany {
                symbol: "main".to_string(),
                module: path.clone(),
            });
        }

        let f_main = candidates.pop().unwrap();
        Ok(f_main)
    }

    /// Resolve module based on idpath
    pub fn resolve_module(&self, idpath: &[String]) -> Result<&Module<'a>, FindError> {
        self.module_info
            .get(idpath)
            .ok_or_else(|| FindError::ModuleNotFound(idpath.to_vec()))
    }
}

impl<'a> CompilerContext<'a> {
    /// Checks, whether supplied function call is a buildin function
    /// buildin functions behave quite different from values in some respect,
    /// which is fine. They will be hidden away in the stdlib.
    /// Returns None, if the supplied function call does not address a buildin function.
    fn check_buildin_func(
        &'a self,
        func: &ast::expr::FunctionCall,
        args: &[Value<'a>],
    ) -> Option<Result<Value<'a>, EvalError>> {
        if func.function_name.value.len() != 1 {
            return None;
        }

        let fname = func.function_name.value[0].value;

        if !fname.starts_with("buildin_") && !fname.starts_with("Buildin_") {
            return None;
        }

        // cut off "buildin_" or "Buildin_"
        let shortened = &fname["buildin_".len()..];

        let res = match shortened {
            "str_concat" => self.buildin_str_concat(args),
            "identity" => self.buildin_identity(args),
            "readline" => self.buildin_readline(args),
            "print" => self.buildin_print(args),

            _ => Err(EvalError::WrongBuildin {
                found: fname.to_string(),
            }),
        };

        Some(res)
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

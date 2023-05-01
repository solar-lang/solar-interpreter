use crate::util;
use crate::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

use solar_parser::{ast, ast::expr::FullExpression, Ast};

pub struct InterpreterContext {
    pub stdout: Mutex<Box<dyn std::io::Write>>,
    pub stdin: Mutex<Box<dyn std::io::Read>>,
    pub global_scope: HashMap<String, Value>,
}

pub struct Context<'a> {
    pub sources: HashMap<Vec<String>, Ast<'a>>,
    pub interpreter_ctx: InterpreterContext,
}

impl<'a> Context<'a> {
    fn resolve_ast(&self, path: &[String]) -> Option<&Ast<'a>> {
        self.sources.get(path)
    }

    fn check_buildin_func(
        &self,
        func: &ast::expr::FunctionCall<'a>,
        args: &[Value],
    ) -> Option<Result<Value, EvalError>> {
        if func.function_name.value.len() != 1 {
            return None;
        }

        let fname = func.function_name.value[0].value;

        if !fname.starts_with("buildin_") || !fname.starts_with("Buildin_") {
            return None;
        }

        let res = match fname {
            "buildin_readline" => self.buildin_readline(args),
            "buildin_print" => self.buildin_print(args),

            _ => Err(EvalError {
                span: func.span.to_string(),
                kind: ErrorType::WrongBuildin {
                    found: fname.to_string(),
                },
            }),
        };

        Some(res)
    }

    fn buildin_print(&self, args: &[Value]) -> Result<Value, EvalError> {
        // allowed overloadings:
        // [String]
        // []
        for arg in args {
            let mut out = self.interpreter_ctx.stdout.lock().expect("lock stdout");

            write!(out, "{arg}").expect("write to stdout");
        }

        Ok(Value::Void)
    }

    fn buildin_readline(&self, args: &[Value]) -> Result<Value, EvalError> {
        // allowed overloadings:
        // [String]
        // []
        if !args.is_empty() {
            if args.len() > 1 {
                panic!("Expected 1 argument of type string to buildin_readline");
            }

            let s = if let Value::String(s) = &args[0] {
                s
            } else {
                panic!("Expected argument to buildin_readline to be of type string");
            };

            let mut out = self.interpreter_ctx.stdout.lock().expect("lock stdout");

            write!(out, "{s}").expect("write to stdout");
        }

        let mut r = self
            .interpreter_ctx
            .stdin
            .lock()
            .expect("lock standart input");
        let mut s = Vec::new();

        loop {
            // read exactly one character
            let mut buf = [0];
            r.read_exact(&mut buf).expect("read from input");
            // grab buffer as character
            let b = buf[0];

            // if the character is new line, skip
            if b == b'\n' {
                break;
            }

            s.push(b)
        }

        let s = String::from_utf8(s).expect("parse stdin as a string");
        Ok(s.into())
    }

    pub fn find_main(&'a self) -> Result<&'a ast::Function<'a>, util::FindError> {
        let path = Vec::new();
        let ast = self.sources.get(&path).unwrap();

        util::find_in_ast(ast, "main")
    }

    pub fn eval_function(
        &self,
        func: &ast::Function<'a>,
        args: &[Value],
    ) -> Result<Value, EvalError> {
        let mut scope = Scope::new();

        // TODO what to do with the type here?
        for ((ident, _ty), val) in func.args.iter().zip(args) {
            scope.push(ident.value, val.clone());
        }

        self.eval(&func.body, &mut scope)
    }

    pub fn eval(&self, expr: &FullExpression, scope: &mut Scope) -> Result<Value, EvalError> {
        match expr {
            FullExpression::Let(expr) => {
                // Insert all let bindings into scope
                // and evaluate their expressions
                for (ident, value) in &expr.definitions {
                    let value = self.eval(value, scope)?;
                    scope.push(ident.value, value)
                }

                // We now have readied the scope and are able to evaluate the body

                let v = self.eval(&expr.body, scope);

                // Now we remove the let bindings from the scope
                for _ in &expr.definitions {
                    scope.pop();
                }

                v
            }

            FullExpression::Expression(ref expr) => match expr as &ast::expr::Expression {
                ast::expr::Expression::FunctionCall(fc) => {
                    let mut args: Vec<Value> = Vec::with_capacity(fc.args.len());
                    for arg in fc.args.iter() {
                        let v = self.eval_sub_expr(&arg.value, scope)?;
                        args.push(v);
                    }

                    if let Some(result) = self.check_buildin_func(fc, &args) {
                        return result;
                    }

                    // Find function name in scope
                    // TODO first check if Type::path contains function name
                    let mut path = util::normalize_path(&fc.function_name);
                    let name = path.pop().unwrap();

                    let ast = self
                        .resolve_ast(&path)
                        // TODO this should be an error
                        .expect("find path of expression");

                    // Find function in AST
                    // TODO first check if it was found before
                    // and use compiled version
                    let func = util::find_in_ast(ast, &name)
                        // TODO should be an error
                        .expect("find method or type");

                    self.eval_function(func, &args)
                }
                ast::expr::Expression::Value(value) => self.eval_sub_expr(value, scope),
            },

            expr => panic!("Unexpected type of expression: {expr:#?}"),
        }
    }

    fn eval_sub_expr(
        &self,
        expr: &ast::expr::Value,
        _scope: &mut Scope,
    ) -> Result<Value, EvalError> {
        use ast::expr::Literal;
        use ast::expr::Value as V;
        match expr {
            V::Literal(lit) => match lit {
                Literal::StringLiteral(_) => {
                    panic!("there is a string here? Should be Interpolated String")
                }
                Literal::Bool { value, .. } => Ok(Value::Bool(*value)),
                Literal::Int(int) => {
                    let i = util::eval_int(int);
                    if let Err(e) = i {
                        return Err(EvalError {
                            // NOTE it would be lovely to have a method to get the line number and row of an AST item.
                            span: int.span.to_string(),
                            kind: e.into(),
                        });
                    }

                    Ok(Value::Int(i.unwrap()))
                }
                Literal::Float(f) => {
                    let f = f.parse::<f64>().expect("float to be in valid f64 form");
                    Ok(Value::Float(f))
                }
            },
            _ => panic!("evaluation not ready"),
        }
    }
}

#[derive(Debug, Clone, Default)]
/// Logical Scope, optimized for small number of entries.
/// Made so pushing and popping works fine.
pub struct Scope {
    values: Vec<(String, Value)>,
}

impl Scope {
    pub fn new() -> Self {
        Scope::default()
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.iter().rfind(|(n, _)| n == name).map(|(_, v)| v)
    }

    pub fn push(&mut self, name: &str, value: Value) {
        self.values.push((name.to_string(), value));
    }

    /// Pops the most recent value out of the scope.
    /// Popping of an empty scope is considered a programming error
    /// and results in a panic.
    pub fn pop(&mut self) -> Value {
        self.values.pop().expect("find value in local scope").1
    }

    pub fn assert_empty(&self) {
        if self.values.is_empty() {
            return;
        }

        panic!("Scope is supposed to be empty");
    }
}

#[derive(Debug, Error)]
pub struct EvalError {
    span: String,
    kind: ErrorType,
}

#[derive(Debug, Error)]
enum ErrorType {
    IntConversion(#[from] std::num::ParseIntError),
    WrongBuildin { found: String },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error at: {}:\n{}", self.span, self.span)
    }
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::IntConversion(e) => e.fmt(f),
            ErrorType::WrongBuildin { found } => {
                write!(f, "only buildin methods are allowed to start with buildin_ or Buildin_.\n Found {found}.")
            }
        }
    }
}

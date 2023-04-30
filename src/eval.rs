use crate::util;
use crate::Value;
use std::collections::HashMap;

use solar_parser::{ast, ast::expr::FullExpression, Ast};

pub struct InterpreterContext {
    pub stdout: Box<dyn std::io::Write>,
    pub stdin: Box<dyn std::io::Read>,
    pub global_scope: HashMap<String, Value>,
}

pub struct Context<'a> {
    pub sources: HashMap<Vec<String>, Ast<'a>>,
    pub ictx: InterpreterContext,
}

impl<'a> Context<'a> {
    fn resolve_ast(&self, path: &[String]) -> Option<&Ast<'a>> {
        self.sources.get(path)
    }

    pub fn eval_function(&self, func: &ast::Function<'a>, args: &[Value]) -> Value {
        let mut scope = Scope::new();

        // TODO what to do with the type here?
        for ((ident, _ty), val) in func.args.iter().zip(args) {
            scope.push(ident.value, val.clone());
        }

        self.eval(&func.body, &mut scope)
    }

    pub fn eval(&self, expr: &FullExpression, scope: &mut Scope) -> Value {
        match expr {
            FullExpression::Let(expr) => {
                // Insert all let bindings into scope
                // and evaluate their expressions
                for (ident, value) in &expr.definitions {
                    let value = self.eval(value, scope);
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
                    let mut args: Vec<Value> = fc
                        .args
                        .iter()
                        .map(|arg| self.eval_sub_expr(&arg.value, &mut scope))
                        .collect();

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
                    let func = util::find_in_ast(&ast, &name)
                        // TODO should be an error
                        .expect("find method or type");

                    self.eval_function(&func, &args)
                }
                ast::expr::Expression::Value(value) => self.eval_sub_expr(&value, &mut scope),
            },

            expr => panic!("Unexpected type of expression: {expr:#?}"),
        }
    }

    fn eval_sub_expr(&self, expr: &ast::expr::Value, scope: &mut Scope) -> Value {}
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

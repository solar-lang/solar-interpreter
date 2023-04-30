mod value;
use std::collections::HashMap;
use value::Value;

use solar_parser::{
    ast::{self, expr::FullExpression},
    Ast,
};

fn main() {
    use solar_parser::Parse;

    let path = std::env::args()
        .nth(1)
        .expect("find filename as first argument");
    let source_code = std::fs::read_to_string(path).expect("read input file");
    let ast = {
        let (rest, ast) = Ast::parse_ws(&source_code).expect("parse source code");
        if !rest.trim_start().is_empty() {
            eprintln!("failed to parse source code. Error at: \n{rest}");
            std::process::exit(0);
        }

        ast
    };

    // Find main function
    let f_main = find(&ast, "main");

    let ctx = Context {
        stdout: Box::new(std::io::stdout()),
        global_scope: HashMap::new(),
    };
}

struct Context {
    stdout: Box<dyn std::io::Write>,
    global_scope: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
/// Logical Scope, optimized for small number of entries.
/// Made so pushing and popping works fine.
pub struct Scope {
    values: Vec<(String, Value)>
}

impl Scope {
    pub fn get(&self ,name: &str) -> Option<&Value> {
        self.values.iter().rfind(|(n, _)| n == name)
            .map(|(_, v)| v)
    }

    pub fn push(&mut self ,name: &str, value: Value) {
        self.values.push((name.to_string(), value));
    }

    /// Pops the most recent value out of the scope.
    /// Popping of an empty scope is considered a programming error
    /// and results in a panic.
    pub fn pop(&mut self ,name: &str) -> Value {
        self.values.pop().expect("find value in local scope").1
    }

    pub fn assert_empty(&self) {
        if self.values.is_empty() {
            return;
        }

        panic!("Scope is supposed to be empty");
    }
}

impl Context {
    pub fn eval_function<'a>(&self, func: &ast::Function<'a>, args: &[Value]) -> Value {
        let mut scope = HashMap::new();

        // TODO what to do with the type here?
        for ((ident, _ty), val) in func.args.into_iter().zip(args) {
            scope.insert(ident.value.to_string(), val.clone());
        }

        self.eval(&func.body, scope)
    }

    pub fn eval<'a>(&self, expr: &FullExpression<'a>, scope: HashMap<String, Value>) -> Value {
        match expr {
            FullExpression::Let(expr) => {
                for (ident, value) &expr.definitions {
                    
                }
            }
            unknown => panic!("Unexpected type of expression: {:#?}", unknown),
        }
    }
}

#[derive(Debug, Clone)]
enum FindError {
    NotFound(String),
}

impl std::fmt::Display for FindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindError::NotFound(name) => write!(f, "Function {name} not found"),
        }
    }
}

fn find<'a>(ast: &'a Ast<'a>, func_name: &str) -> Result<&'a ast::Function<'a>, FindError> {
    for i in &ast.items {
        if let solar_parser::ast::body::BodyItem::Function(f) = i {
            if f.name != func_name {
                continue;
            }

            // TODO check compatible types here

            return Ok(f);
        }
    }

    Err(FindError::NotFound(func_name.to_string()))
}

struct CompilationContext {}

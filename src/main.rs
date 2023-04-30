mod value;

use solar_parser::{ast, Ast};

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

    let f_main = find(&ast, "main");
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

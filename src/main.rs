mod eval;
mod util;
mod value;
use eval::*;
use std::collections::HashMap;
use value::Value;

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

    // Find main function
    let f_main = util::find_in_ast(&ast, "main").expect("find main function");

    let ctx = Context {
        sources: [(Vec::new(), ast)].into_iter().collect(),
        ictx: InterpreterContext {
            stdout: Box::new(std::io::stdout()),
            stdin: Box::new(std::io::stdin()),
            global_scope: HashMap::new(),
        },
    };

    ctx.eval_function(f_main, &[]);
}

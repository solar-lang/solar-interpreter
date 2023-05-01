mod eval;
mod util;
mod value;
use eval::*;
use std::collections::HashMap;
use std::sync::Mutex;
use value::Value;

use solar_parser::Ast;

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

    let ctx = Context {
        sources: [(Vec::new(), ast)].into_iter().collect(),
        interpreter_ctx: InterpreterContext {
            stdout: Mutex::new(Box::new(std::io::stdout())),
            stdin: Mutex::new(Box::new(std::io::stdin())),
            global_scope: HashMap::new(),
        },
    };

    // Find main function
    let f_main = ctx.find_main().expect("find main function");

    ctx.eval_function(f_main, &[]).expect("run main function");
}

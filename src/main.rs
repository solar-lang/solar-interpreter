#![feature(string_leak)]
mod eval;
pub mod id;
mod project;
mod util;
mod value;

use eval::CompilerContext;

use project::{read_all_projects, read_modules};

fn main() {
    let fsroot = std::env::args().nth(1).unwrap_or(".".to_string());
    let project_info = read_all_projects(&fsroot).expect("read in solar project and dependencies");
    let modules = read_modules(&project_info).expect("open and parse solar files");

    let ctx = CompilerContext::with_default_io(&project_info, modules);
    let f_main = ctx.find_target_main().expect("find main function");

    let result = ctx.eval_symbol(f_main, &[]).expect("evaluate code");

    eprintln!("\n{result:?}");
}

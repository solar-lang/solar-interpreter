#![feature(string_leak)]
mod eval;
mod project;
mod util;
mod value;

use eval::CompilerContext;

use project::{read_all_projects, read_modules};

use value::Value;

fn main() {
    let fsroot = std::env::args().nth(1).unwrap_or(".".to_string());
    let project_info = read_all_projects(&fsroot).expect("read in solar project and dependencies");
    let modules = read_modules(&project_info).expect("open and parse solar files");

    let ctx = CompilerContext::with_default_io(project_info, modules);
}

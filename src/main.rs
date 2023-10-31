// #![feature(string_leak)]
mod eval;
pub mod id;
mod project;
mod types;
mod util;
mod value;

use project::{read_all_projects, read_modules};

use eval::CompilerContext;

use crate::types::buildin::link_buildin_types;

fn main() {
    let fsroot = std::env::args().nth(1).unwrap_or(".".to_string());
    let project_info = read_all_projects(&fsroot).expect("read in solar project and dependencies");
    let modules = read_modules(&project_info).expect("open and parse solar files");

    let (_, _) = link_buildin_types(&project_info, &modules);

    let ctx = CompilerContext::with_default_io(&project_info, modules);

    let f_main = ctx.find_target_main().expect("find main function");

    let result = ctx.eval_symbol(f_main, &[]).expect("evaluate code");

    /* TODO
        There's a need now, to resolve types.
        That means filling the CompilerContext::types,
        which in turn yields TypeIDs, that we can use.
        TypeIDs are required to build SSIDs.
        If we have SSIDs, we can create a HotelMap to yield
        concrete function implementations.
        E.g. Function(..Args) -> AST
        and further we can then derive concrete ByteCode already!
        E.g. Function(..Args) -> ByteCode
    */

    eprintln!("\n{result:?}");

    eprintln!("\n{:?}", ctx.types);
}

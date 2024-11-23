// #![feature(string_leak)]
pub mod mir;
mod compilation;
pub mod id;
mod project;
mod types;
mod util;
mod value;

use project::{read_all_projects, read_modules};

use compilation::CompilerContext;

fn main() {
    let fsroot = std::env::args().nth(1).unwrap_or(".".to_string());
    let project_info = read_all_projects(&fsroot).expect("read in solar project and dependencies");
    let modules = read_modules(&project_info).expect("open and parse solar files");

    let ctx = CompilerContext::with_default_io(&project_info, modules);

    let f_main = ctx.find_target_main().expect("find main function");

    // TODO instead call resolve_symbol(f_main, &[]) -> FunctionID
    // -> why?
    let function_id = ctx.compile_symbol(f_main, &[]).expect("compile code");

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

    eprintln!("\n{function_id:#?}");

    // eprintln!("\n{:#?}", ctx.types);

    for (k,_i, v) in ctx.functions.read().unwrap().iter(){
        let k = k.0.0.join(".") + &format!(".{}", k.0.1);
        eprintln!("{}:\n{:#?}\n", k, v);
    } 

}

mod eval;
mod project;
mod util;
mod value;
use eval::*;
use std::sync::Mutex;
use value::Value;

use solar_parser::Ast;

fn main() {
    let cfg =
        project::SolarConfig::read("solar.yaml").expect("read solar config in current directory");

    let basepath = cfg.basepath();
    dbg!(basepath);

    // read all .sol files in ./ as root
    // collect them as
    // modules := {}
    // deps := cfg.deps() # map. e.g. "std" => [std(solar-lang), 0.0.1]
    // foreach _file, path, fullpath of ./**/*.sol:
    //    modulepath := path.split("/")
    //    module := basepath ++ modulepath
    //    if module not in modules:
    //        modules[module] = []
    //
    //    # deps are needed here, to know which VERSION the deps in this file are supposed to resolve to
    //    modules[module].append(FileContext::from_file(module, file=fullpath, deps=deps))
}

/*
    ProjectConfig
        # (only) for the current project (and local libs), an publisher will be ommitted
        basepath: ["nice-gui()", "2.1.3"],
        depmap:
            "std" => ["std(solar-lang)", "0.0.1"]

    ModuleConfig
        project_id: usize
        # include projectconfig reference?
        # include basepath?
        # path: ["types"]
        files: [FileContext("string.sol"), FileContext("string_util.sol")]
        # TODO static_functions: Array<{name: String, args: Vec<Type>, ret: Type}>

    FileContext    (owned by ModuleConfig)
        # notice, how the function names are not path of their path (e.g. module identifier)
        imports:
            # e.g.
            # use @std.string.concat
            "concat" => ["std(solar-lang)", "0.0.1", "string"]
            # use util.(flipbits, redraw)
            "flipbits" => ["nice-gui()", "0.0.1", "util" ]
            "redraw" => ["nice-gui()", "0.0.1", "util"]
        ast: Ast
        # TODO compiled functions here?
        # or on module level?
        # ==> on module level, because +
            1.) we need file distinction only for resolving imports
            2.) we have a flat hierarchy inside a module.

}

*/

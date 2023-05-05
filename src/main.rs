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

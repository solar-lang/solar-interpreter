mod eval;
mod project;
mod util;
mod value;
use hotel::HotelMap;
use project::{IdPath, Project};
use value::Value;

fn target_id() -> IdPath {
    vec!["self".to_string()]
}

fn main() {
    let fsroot = std::env::args().nth(1).unwrap_or(".".to_string());
    let mut projects = HotelMap::new();
    let p = Project::open(&fsroot, target_id());
    fn insert_all(p: Project, projects: &mut HotelMap<Vec<String>, Project>) {
        for dep in p.config.deps() {
            let path = dep.basepath();
            // skip project, if we have already read it.
            if projects.contains(&path) {
                continue;
            }

            let dir = dep.dir();
            let p = Project::open(&dir, path);
            insert_all(p, projects);
        }

        projects.insert(p.basepath.clone(), p);
    }

    insert_all(p, &mut projects);

    for (project_id, p) in projects.iter_values() {
        p.read_all(project_id);
    }

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

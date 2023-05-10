#![feature(string_leak)]
mod eval;
mod project;
mod util;
mod value;
use anyhow::Result;
use hotel::HotelMap;
use project::Project;
use util::IdPath;
use value::Value;

fn read_all_projects(fsroot: &str) -> Result<HotelMap<IdPath, Project>> {
    let mut projects = HotelMap::new();
    let p = Project::open(fsroot, util::target_id())?;

    fn insert_all(p: Project, projects: &mut HotelMap<Vec<String>, Project>) -> Result<()> {
        for dep in p.config.deps() {
            let path = dep.basepath();
            // skip project, if we have already read it.
            if projects.contains(&path) {
                continue;
            }

            let dir = dep.dir();
            let p = Project::open(&dir, path)?;
            insert_all(p, projects)?;
        }

        projects.insert(p.basepath.clone(), p);

        Ok(())
    }

    insert_all(p, &mut projects)?;

    Ok(projects)
}

fn main() {
    let fsroot = std::env::args().nth(1).unwrap_or(".".to_string());
    let projects = read_all_projects(&fsroot).expect("read in solar project and dependencies");

    for (project_id, p) in projects.iter_values() {
        dbg!(&p);
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

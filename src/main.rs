#![feature(string_leak)]
mod eval;
mod project;
mod util;
mod value;
use std::collections::HashMap;

use anyhow::Result;
use eval::CompilerContext;
use hotel::HotelMap;
use project::{Module, Project};
use util::IdPath;
use value::Value;

pub type ProjectInfo = HotelMap<IdPath, Project>;

pub type GlobalModules<'a> = HashMap<IdPath, Module<'a>>;

fn read_all_projects(fsroot: &str) -> Result<ProjectInfo> {
    let mut projects = HotelMap::new();
    let p = Project::open(fsroot, util::target_id())?;

    fn insert_all(p: Project, projects: &mut HotelMap<IdPath, Project>) -> Result<()> {
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

/// create global mapping of ModulePaths to Modules
/// i.e. across all dependencies and sub-dependencies
fn read_modules(projects: &ProjectInfo) -> anyhow::Result<GlobalModules> {
    let mut modules = HashMap::new();

    for (project_id, project) in projects.iter_values() {
        let symbol_table = project.read_all(project_id)?;

        for (sym, path) in symbol_table.into_iter() {
            symbol_table.insert(sym, path);
        }
    }

    Ok(modules)
}

fn main() {
    let fsroot = std::env::args().nth(1).unwrap_or(".".to_string());
    let project_info = read_all_projects(&fsroot).expect("read in solar project and dependencies");
    let modules = read_modules(&project_info).expect("open and parse solar files");

    let ctx = CompilerContext {
        modules,
        project_info,
        interpreter_ctx: eval::InterpreterContext::new(),
    };
}

use super::Module;
/// This file contains code
/// for reading in dependencies and libraries.
/// and resolving their imports.
use crate::project::SolarConfig;
use std::collections::HashMap;
use walkdir::WalkDir;

pub type IdPath = Vec<String>;

/// Contains information on a project,
/// and how to resolve imports inside a project.
#[derive(Debug)]
pub struct Project {
    /// Unique Id of this project
    pub basepath: IdPath,
    /// Root in filesystem
    pub fsroot: String,

    /// mapping needed to resolve imports in a project to actual
    /// dependencies
    pub dep_map: HashMap<String, IdPath>,

    /// Solarconfig of this project
    pub config: SolarConfig,
}

impl Project {
    pub fn open(
        // The root is the root of the project
        // in the filetree
        fsroot: &str,
        // the basepath is the unique identifier for this project.
        // For libraries it is supposed to match the root.
        basepath: IdPath,
    ) -> Project {
        let solarfile = format!("{fsroot}/solar.yaml");

        // solar config file of the project
        let Ok(config) = SolarConfig::read(&solarfile) else {
        panic!("expected to find solar config in {solarfile}")
    };

        let fsroot = fsroot.to_string();

        let dep_map = config
            .deps()
            .into_iter()
            .map(|d| {
                let value = d.basepath();
                let key = d.name;
                (key, value)
            })
            .collect();

        Project {
            basepath,
            fsroot,
            dep_map,
            config,
        }
    }

    /// Reads and parses
    /// all solarfiles out of all modules
    /// into memory.
    /// the project id is supposed to be a reference to this exact [Project]
    pub fn read_all(&self, project_id: usize) -> HashMap<IdPath, Module> {
        let mut map = HashMap::new();

        for entry in WalkDir::new(&self.fsroot) {
            let Ok(entry) = entry else {
                    eprintln!("error walking directory: {entry:?}");
                    continue;
                };

            if !entry.file_type().is_file() {
                continue;
            }

            let filename = entry.file_name().to_str().expect("read filename");
            if !filename.ends_with(".sol") {
                continue;
            }

            let path = entry.path();
            // We need to strip the path,
            // because we don't care about the root file system
            let relativepath = path
                .strip_prefix(&self.fsroot)
                .expect("to strip common prefix of filepath");
            let relativepath = relativepath.iter().collect::<Vec<_>>();
            dbg!(relativepath);
        }

        map
    }
}

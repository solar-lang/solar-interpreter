use super::IdPath;
use anyhow::Result;
use solar_parser::Ast;
use std::collections::HashMap;

pub struct Module<'a> {
    // NOTE u32 might be better
    pub project_id: usize,
    /// Set of all file inside this module
    pub files: Vec<FileInfo<'a>>,
    // compiled_functions on module level, because
    //     1.) we need file distinction only for resolving imports
    //     2.) we have a flat hierarchy inside a module.
    // TODO compiled_functions: {name => (args, ret, body)}
}

impl<'a> Module<'a> {
    pub fn new(project_id: usize) -> Self {
        Self {
            project_id,
            files: Vec::new(),
        }
    }

    pub fn add_file(&mut self, file: FileInfo<'a>) {
        self.files.push(file);
    }
}

pub struct FileInfo<'a> {
    // NOTE this might be redundant
    pub filename: String,
    pub imports: HashMap<String, IdPath>,
    pub ast: Ast<'a>,
}

impl<'a> FileInfo<'a> {
    fn from_code(depmap: &HashMap<String, IdPath>, content: &'a str) -> Result<Self> {
        use solar_parser::Parse;
        let (rest, ast) = solar_parser::Ast::from_source_code(content)?;
        // TODO continue here
    }
}

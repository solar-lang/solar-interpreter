use super::IdPath;
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

pub struct FileInfo<'a> {
    // NOTE this might be redundant
    pub filename: String,
    pub imports: HashMap<String, IdPath>,
    pub ast: Ast<'a>,
}

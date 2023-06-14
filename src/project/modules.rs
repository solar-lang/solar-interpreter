use crate::eval::{CompilerContext, FunctionContext};
use crate::id::{IdModule, SymbolId};
use crate::util::IdPath;
use solar_parser::ast::import::Selection;
use solar_parser::{ast, Ast};
use std::collections::HashMap;
use thiserror::Error;

pub type SymbolResolver = HashMap<String, Vec<IdPath>>;

#[derive(Debug)]
pub struct Module<'a> {
    // NOTE u32 might be better
    // TODO we don't need that, after having resolved all import tables at ast creation time.
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

    pub fn find(&'a self, symbol: &str, idmodule: IdModule) -> Result<Vec<SymbolId>, FindError> {
        let mut v = Vec::new();

        for (idfile, fileinfo) in &self.files.iter().enumerate() {
            let ast = &fileinfo.ast;
            for (iditem, i) in ast.items.iter().enumerate() {
                match i {
                    ast::body::BodyItem::Function(f) if f.name == symbol => {
                        v.push((idmodule.clone(), idfile as u32, iditem as u32));
                    }
                    ast::body::BodyItem::TypeDecl(t) if t.name == symbol => {
                        panic!("Resolver can't yet handle types")
                    }

                    _ => continue,
                    // Tests don't have names,
                    // ast::body::BodyItem::Test(_) => continue,
                    // Let bindings are resolved into the global scope
                    // ast::body::BodyItem::Let(_) => continue,
                }
            }
        }

        if v.is_empty() {
            return Err(FindError::NotFound(symbol.to_string()));
        }

        Ok(v)
    }
}

#[derive(Debug, Clone, Error)]
pub enum FindError {
    NotFound(String),
    ModuleNotFound(IdPath),
    TooMany { symbol: String, module: IdPath },
}

impl std::fmt::Display for FindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindError::NotFound(name) => write!(f, "Function (or Type) {name} not found"),
            Self::ModuleNotFound(path) => write!(f, "Module {path:?} not found"),
            Self::TooMany { symbol, module } => write!(
                f,
                "found too many candidates for symbol '{symbol}' in module {module:?}. Expected to find just 1"
            ),
        }
    }
}

#[derive(Debug)]
pub struct FileInfo<'a> {
    // NOTE this might be redundant
    pub filename: String,

    /// Maps individual symbols (e.g. `length`) to paths,
    /// where they should be found in (e.g. std/0.0.1/string/).
    /// It may be, that multiple locations apply.
    /// e.g.
    ///    use std.string.length
    ///    use std.array.length
    /// is valid, expected
    /// and will require resolving from multiple locations.
    pub imports: SymbolResolver,
    pub ast: Ast<'a>,
}

#[derive(Debug, Error)]
pub enum ResolveError<'a> {
    LibNotInDeps {
        // TODO include location...
        // but how do I want to do it project wide?
        libname: String,
    },
    ParseErr(ast::NomErr<'a>),
}

impl<'a> From<ast::NomErr<'a>> for ResolveError<'a> {
    fn from(value: ast::NomErr<'a>) -> Self {
        ResolveError::ParseErr(value)
    }
}

impl std::fmt::Display for ResolveError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::LibNotInDeps { libname } => write!(
                f,
                "imported libraries '{libname}' not found in dependencies"
            ),
            ResolveError::ParseErr(e) => e.fmt(f),
        }
    }
}

impl<'a> FileInfo<'a> {
    pub fn from_code(
        filename: String,
        depmap: &HashMap<String, IdPath>,
        basepath: &IdPath,
        content: &'a str,
    ) -> Result<Self, ResolveError<'a>> {
        // read in file and parse AST.
        let ast = Ast::from_source_code(content)?;

        // build up lookup table to resolve imported symbols.
        let imports = resolve_imports(&ast, depmap, basepath)?;

        Ok(FileInfo {
            filename,
            imports,
            ast,
        })
    }
}

/// Resolve all imports from the ast to their global symbols for later lookup.
fn resolve_imports<'a>(
    ast: &Ast<'a>,
    depmap: &HashMap<String, IdPath>,
    basepath: &IdPath,
) -> Result<SymbolResolver, ResolveError<'a>> {
    let mut imports = HashMap::new();

    for import in ast.imports.iter() {
        // the ID path might be from a library, or from this project.
        // Here we switch based on that.
        let mut path = if import.is_lib {
            // now let's resolve this relative import (e.g. std.types.string) to an absolute path
            // that we can use as global identifier.

            // get the name of the library
            let lib = import.path[0].value;
            // resolve correct version from library
            let lib_path = depmap
                .get(lib)
                // if we can't find the symbol inside the dependencies, it's an error
                .ok_or_else(|| ResolveError::LibNotInDeps {
                    libname: lib.to_string(),
                })?;

            // append rest of the import path to the absolute path we just created
            let mut path = lib_path.clone();
            path.extend(import.path[1..].iter().map(String::from));
            path
        } else {
            // e.g.
            // use models.customer

            // basepath is the currently active project
            // and we just concatenate the import to this base path
            basepath
                .iter()
                .cloned()
                .chain(import.path.iter().map(String::from))
                .collect()
        };

        match &import.items {
            Selection::All => {
                unimplemented!("{}\n{}\n{}",
                "found '..' selection.",
                "Needs lookup for all symbols in a library.",
                "Will need to happen eventually anyway, in order to check that every import is valid (and public)"
            );
            }
            Selection::This => {
                // the last symbol of the path was the concrete import item.
                // just pop it of the path, and we're golden.
                let symbol = path
                    .pop()
                    .expect("Concrete symbol to be at the end of import path");
                imports.entry(symbol).or_insert_with(Vec::new).push(path);
            }
            Selection::Items(s) => {
                // Importing multiple symbols from this library.
                // Add them all!
                for symbol in s.iter() {
                    let symbol = symbol.value.to_string();
                    imports
                        .entry(symbol)
                        .or_insert_with(Vec::new)
                        .push(path.clone());
                }
            }
        }
    }

    Ok(imports)
}

/// Context containing all needed information
/// for evalutating a specific function
#[derive(Debug, Clone, Copy)]
// TODO remove
pub struct FunctionInfo<'a> {
    /// Information about the file,
    /// such as the files ast, filename, and
    /// the import table.
    pub file_info: &'a FileInfo<'a>,

    /// Info about the module this file can be found in.
    /// Needed to resolve symbols within the function.
    pub module: &'a Module<'a>,

    /// AST of the function
    pub ast: &'a ast::Function<'a>,
}

impl<'a> FunctionInfo<'a> {
    pub fn new(
        file_info: &'a FileInfo<'a>,
        module: &'a Module<'a>,
        ast: &'a ast::Function<'a>,
    ) -> Self {
        Self {
            file_info,
            module,
            ast,
        }
    }

    /// Returns a number uniquely identifying this specific function info.
    /// Implemented using the memory location of the source codes ast of the function.
    pub fn unique_id(&self) -> usize {
        self.ast.span.as_ptr() as usize
    }

    pub fn ctx<'ctx>(self, ctx: &'ctx CompilerContext<'a>) -> FunctionContext<'ctx, 'a> {
        FunctionContext { ctx, info: self }
    }
}

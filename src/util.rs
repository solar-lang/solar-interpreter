use solar_parser::{ast, ast::identifier::IdentifierPath, Ast};

pub(crate) fn normalize_path(path: &IdentifierPath) -> Vec<String> {
    // TODO find path[0] in imports
    // and append this import to the start
    // NOTE: this is context dependent.
    // e.g. associated methods don't need to be in scope.

    path.value
        .into_iter()
        .map(|i| i.value.to_string())
        .collect()
}

#[derive(Debug, Clone)]
enum FindError {
    NotFound(String),
}

impl std::fmt::Display for FindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindError::NotFound(name) => write!(f, "Function (or Type) {name} not found"),
        }
    }
}

pub fn find_in_ast<'a>(ast: &'a Ast<'a>, item: &str) -> Result<&'a ast::Function<'a>, FindError> {
    for i in &ast.items {
        match &i {
            ast::body::BodyItem::Function(f) if f.name == item => {
                // TODO check compatible types here

                return Ok(f);
            }
            ast::body::BodyItem::TypeDecl(t) if t.name == item => {
                panic!("Resolver can't yet handle types")
            }

            // Tests don't have names,
            ast::body::BodyItem::Test(_) => continue,
            // Let bindings are resolved into the global scope
            ast::body::BodyItem::Let(_) => continue,
        }
    }

    Err(FindError::NotFound(item.to_string()))
}

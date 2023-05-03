use solar_parser::{ast, ast::identifier::IdentifierPath, Ast};
use thiserror::Error;

pub(crate) fn normalize_path(path: &IdentifierPath) -> Vec<String> {
    // TODO find path[0] in imports
    // and append this import to the start
    // NOTE: this is context dependent.
    // e.g. associated methods don't need to be in scope.

    path.value.iter().map(|i| i.value.to_string()).collect()
}

#[derive(Debug, Clone, Error)]
pub enum FindError {
    NotFound(String),
}

impl std::fmt::Display for FindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindError::NotFound(name) => write!(f, "Function (or Type) {name} not found"),
        }
    }
}

// TODO return Vec of Functions!
pub fn find_in_ast<'a>(ast: &'a Ast<'a>, item: &str) -> Result<&'a ast::Function<'a>, FindError> {
    for i in &ast.items {
        match i {
            ast::body::BodyItem::Function(f) if f.name == item => {
                // TODO check compatible types here

                return Ok(f);
            }
            ast::body::BodyItem::TypeDecl(t) if t.name == item => {
                panic!("Resolver can't yet handle types")
            }

            _ => continue,
            // Tests don't have names,
            // ast::body::BodyItem::Test(_) => continue,
            // Let bindings are resolved into the global scope
            // ast::body::BodyItem::Let(_) => continue,
        }
    }

    Err(FindError::NotFound(item.to_string()))
}

pub(crate) fn eval_int(
    int: &ast::expr::literal::Int,
) -> Result<crate::value::Int, std::num::ParseIntError> {
    use ast::expr::literal::IntTypeSuffix as Ty;
    let ty = int.type_suffix.unwrap_or(Ty::Int);
    let radix = int.radix as u32;
    let i = match ty {
        Ty::Int => crate::value::Int::Int64(i64::from_str_radix(int.digits, radix)?),
        Ty::Int32 => crate::value::Int::Int32(i32::from_str_radix(int.digits, radix)?),
        Ty::Int16 => crate::value::Int::Int16(i16::from_str_radix(int.digits, radix)?),
        Ty::Int8 => crate::value::Int::Int8(i8::from_str_radix(int.digits, radix)?),
        Ty::Uint => crate::value::Int::Uint64(u64::from_str_radix(int.digits, radix)?),
        Ty::Uint32 => crate::value::Int::Uint32(u32::from_str_radix(int.digits, radix)?),
        Ty::Uint16 => crate::value::Int::Uint16(u16::from_str_radix(int.digits, radix)?),
        Ty::Uint8 => crate::value::Int::Uint8(u8::from_str_radix(int.digits, radix)?),
    };

    Ok(i)
}

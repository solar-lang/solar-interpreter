use solar_parser::{ast, ast::identifier::IdentifierPath, Ast};
use thiserror::Error;

use crate::project::Module;

/// Denotes an global identifier used to resolve
/// modules and symbols across libraries and versions of libraries.
pub type IdPath = Vec<String>;

/// The IdPath (that is, the common prefix)
/// of all symbols inside the current project
/// (e.g. the one with "./solar.yaml")
pub fn target_id() -> IdPath {
    vec!["self".to_string()]
}

pub(crate) fn normalize_path(path: &IdentifierPath) -> Vec<String> {
    // TODO find path[0] in imports
    // and append this import to the start
    // NOTE: this is context dependent.
    // e.g. associated methods don't need to be in scope.

    path.value.iter().map(|i| i.value.to_string()).collect()
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

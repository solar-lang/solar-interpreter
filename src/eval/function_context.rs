use solar_parser::ast::{self, expr::FullExpression};

use crate::{
    project::{FunctionInfo, SymbolResolver},
    util::{self, IdPath, Scope},
    value::Value,
};

use super::CompilerContext;

/// Contains all information needed to evaluate a function.
pub struct FunctionContext<'a, 'b> {
    pub ctx: &'a CompilerContext<'a>,
    pub info: FunctionInfo<'b>,
}

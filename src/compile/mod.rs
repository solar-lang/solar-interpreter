use crate::id::{FunctionId, TypeId};

/// Expression with type-information
pub type StaticExpression = (
    Box<Instruction>,
    // TODO make this into an array of typeIDs later
    TypeId,
);

pub enum Instruction {
    Void,
    FunctionCall {
        func: FunctionId,
        args: Vec<(Instruction, TypeId)>,
    },
    /// Get local Variable at address
    GetLocalVar(usize),
    NewLocalVar {
        // name: String,
        value: StaticExpression,
        scope: StaticExpression,
    },
    AssignLocalVar(usize, StaticExpression),
    Return(StaticExpression),

    IfExpr {
        /// Must be of typeId == Boolean
        condition: StaticExpression,
        case_true: StaticExpression,
        case_false: StaticExpression,
    }, // we have differnt kinds of for loops and ifs.
       // Expressions and Statements
       // (loops) Expressions return an Array of static types, that must be the same
}

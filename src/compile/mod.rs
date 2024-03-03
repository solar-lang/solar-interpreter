use crate::id::{FunctionId, TypeId};

/// Expression with type-information
pub struct StaticExpression {
    pub instr: Box<Instruction>,
    // NOTE maybe make this into an array of typeIDs later
    pub ty: TypeId,
}

pub enum Instruction {
    Void,
    FunctionCall {
        func: FunctionId,
        args: Vec<(Instruction, TypeId)>,
    },
    /// Get local Variable at address
    GetLocalVar(usize),
    /// Define a new let binding, that can be referenced later
    NewLocalVar {
        // name: String,
        // within the current scope the index is unique
        var_index: u16,
        // The value the variable will hold.
        var_value: StaticExpression,
        /// The expressions coming after the let binding,
        /// where the variable is in scope.
        body: StaticExpression,
    },
    IfExpr {
        /// Must be of typeId == Boolean
        condition: StaticExpression,
        case_true: StaticExpression,
        case_false: StaticExpression,
    }, // we have differnt kinds of for loops and ifs.
       // Expressions and Statements
       // (loops) Expressions return an Array of static types, that must be the same
}

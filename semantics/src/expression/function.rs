use std::{cell::RefCell, rc::Rc};

use crate::{Block, VariableInfo};

use super::ExprLocalVariable;

/// constructing `function`. not a function object.
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    /// stack offset of arguments
    pub args: Vec<Rc<RefCell<VariableInfo>>>,
    /// if true, this function is variadic
    pub variadic: bool,
    /// function body
    pub body: Block,
}

impl FunctionDefinition {
    pub fn new(
        args: Vec<Rc<RefCell<VariableInfo>>>,
        variadic: bool,
        body: Block,
        // stack_size: usize,
    ) -> Self {
        Self {
            args,
            variadic,
            body,
            // stack_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExprFunctionObject {
    /// when constructing function object, copy upvalues from these sources
    pub upvalues_source: Vec<ExprLocalVariable>,

    /// function definition
    pub definition: FunctionDefinition,
}

impl ExprFunctionObject {
    pub fn new(upvalues_source: Vec<ExprLocalVariable>, definition: FunctionDefinition) -> Self {
        Self {
            upvalues_source,
            definition,
        }
    }
}

use std::{cell::RefCell, rc::Rc};

use super::Block;
use crate::{Expression, VariableInfo};

/// for statement with start, end, step.
#[derive(Clone, Debug)]
pub struct StmtFor {
    // this must be a offset for local variable
    pub control_variable: Rc<RefCell<VariableInfo>>,
    pub start: Expression,
    pub end: Expression,
    pub step: Expression,
    pub block: Block,
}
impl StmtFor {
    pub fn new(
        control_variable: Rc<RefCell<VariableInfo>>,
        start: Expression,
        end: Expression,
        step: Expression,
        block: Block,
    ) -> Self {
        Self {
            control_variable,
            start,
            end,
            step,
            block,
        }
    }
}

/// for statement with generic expressions.
#[derive(Clone, Debug)]
pub struct StmtForGeneric {
    pub control_variables: Vec<Rc<RefCell<VariableInfo>>>,
    /// local variable for iterator
    pub iterator: Rc<RefCell<VariableInfo>>,
    /// local variable for state
    pub state: Rc<RefCell<VariableInfo>>,
    /// local variable for closing value
    pub closing: Rc<RefCell<VariableInfo>>,
    pub expressions: Vec<Expression>,
    pub block: Block,
}
impl StmtForGeneric {
    pub fn new(
        control_variables: Vec<Rc<RefCell<VariableInfo>>>,
        iterator: Rc<RefCell<VariableInfo>>,
        state: Rc<RefCell<VariableInfo>>,
        closing: Rc<RefCell<VariableInfo>>,
        expressions: Vec<Expression>,
        block: Block,
    ) -> Self {
        Self {
            control_variables,
            iterator,
            state,
            closing,
            expressions,
            block,
        }
    }
}

use super::Block;
use crate::Expression;

/// for statement with start, end, step.
#[derive(Clone, Debug)]
pub struct StmtFor {
    pub name: String,
    pub start: Expression,
    pub end: Expression,
    pub step: Expression,
    pub block: Block,
}
impl StmtFor {
    pub fn new(
        name: String,
        start: Expression,
        end: Expression,
        step: Expression,
        block: Block,
    ) -> Self {
        Self {
            name,
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
    pub names: Vec<String>,
    pub expressions: Vec<Expression>,
    pub block: Block,
}
impl StmtForGeneric {
    pub fn new(names: Vec<String>, expressions: Vec<Expression>, block: Block) -> Self {
        Self {
            names,
            expressions,
            block,
        }
    }
}

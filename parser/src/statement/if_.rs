use super::Block;
use crate::Expression;
use crate::Span;

/// else-if fragment for if statements
#[derive(Clone, Debug)]
pub struct StmtElseIf {
    pub condition: Expression,
    pub block: Block,
    pub span: Span,
}
impl StmtElseIf {
    pub fn new(condition: Expression, block: Block, span: Span) -> Self {
        Self {
            condition,
            block,
            span,
        }
    }
    /// get the span of the whole else-if fragment
    pub fn span(&self) -> Span {
        self.span
    }
}

/// if statement
#[derive(Clone, Debug)]
pub struct StmtIf {
    pub condition: Expression,
    pub block: Block,
    pub else_ifs: Vec<StmtElseIf>,
    pub else_block: Option<Block>,
    pub span: Span,
}
impl StmtIf {
    pub fn new(
        condition: Expression,
        block: Block,
        else_ifs: Vec<StmtElseIf>,
        else_block: Option<Block>,
        span: Span,
    ) -> Self {
        Self {
            condition,
            block,
            else_ifs,
            else_block,
            span,
        }
    }
    /// get the span of the whole if statement
    pub fn span(&self) -> Span {
        self.span
    }
}

use super::Block;
use crate::Expression;
use crate::Span;

/// while statement
#[derive(Clone, Debug)]
pub struct StmtWhile {
    pub condition: Expression,
    pub block: Block,
    pub span: Span,
}
impl StmtWhile {
    pub fn new(condition: Expression, block: Block, span: Span) -> Self {
        Self {
            condition,
            block,
            span,
        }
    }
    /// get the span of the whole while statement
    pub fn span(&self) -> Span {
        self.span
    }
}

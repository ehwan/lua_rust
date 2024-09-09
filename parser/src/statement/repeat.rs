use super::Block;
use crate::Expression;
use crate::Span;

/// repeat statement
#[derive(Clone, Debug)]
pub struct StmtRepeat {
    pub block: Block,
    pub condition: Expression,
    pub span: Span,
}
impl StmtRepeat {
    pub fn new(block: Block, condition: Expression, span: Span) -> Self {
        Self {
            block,
            condition,
            span,
        }
    }
    /// get the span of the whole repeat statement
    pub fn span(&self) -> Span {
        self.span
    }
}

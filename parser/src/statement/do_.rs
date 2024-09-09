use super::Block;
use crate::Span;

/// do - end statements block
#[derive(Clone, Debug)]
pub struct StmtDo {
    pub block: Block,
    pub span: Span,
}
impl StmtDo {
    pub fn new(block: Block, span: Span) -> Self {
        Self { block, span }
    }
    /// get the span of the whole do-end block
    pub fn span(&self) -> Span {
        self.span
    }
}

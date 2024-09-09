use crate::Span;

/// break statement
#[derive(Debug, Clone, Copy)]
pub struct StmtBreak {
    pub span: Span,
}
impl StmtBreak {
    pub fn new(span: Span) -> Self {
        Self { span }
    }
    /// get the span of the break statement.
    pub fn span(&self) -> Span {
        self.span
    }
}

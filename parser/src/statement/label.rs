use crate::ExprString;
use crate::Span;

/// label definition
#[derive(Clone, Debug)]
pub struct StmtLabel {
    pub name: ExprString,
    pub span: Span,
}
impl StmtLabel {
    pub fn new(name: ExprString, span: Span) -> Self {
        Self { name, span }
    }
    /// get the span of the whole label definition
    pub fn span(&self) -> Span {
        self.span
    }
}

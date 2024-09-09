use crate::ExprString;
use crate::Span;

/// Goto statement
#[derive(Clone, Debug)]
pub struct StmtGoto {
    pub name: ExprString,
    pub span: Span,
}
impl StmtGoto {
    pub fn new(name: ExprString, span: Span) -> Self {
        Self { name, span }
    }
    /// get the span of the whole goto statement
    pub fn span(&self) -> Span {
        self.span
    }
}

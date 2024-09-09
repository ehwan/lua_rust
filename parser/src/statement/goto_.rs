use crate::Span;
use crate::SpannedString;

/// Goto statement
#[derive(Clone, Debug)]
pub struct StmtGoto {
    pub name: SpannedString,
    pub span: Span,
}
impl StmtGoto {
    pub fn new(name: SpannedString, span: Span) -> Self {
        Self { name, span }
    }
    /// get the span of the whole goto statement
    pub fn span(&self) -> Span {
        self.span
    }
}

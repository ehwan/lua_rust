use crate::Span;
use crate::SpannedString;

/// label definition
#[derive(Clone, Debug)]
pub struct StmtLabel {
    pub name: SpannedString,
    pub span: Span,
}
impl StmtLabel {
    pub fn new(name: SpannedString, span: Span) -> Self {
        Self { name, span }
    }
    /// get the span of the whole label definition
    pub fn span(&self) -> Span {
        self.span
    }
}

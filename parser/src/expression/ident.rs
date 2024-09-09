use crate::Span;
use crate::SpannedString;
use crate::Token;

/// just identifier
#[derive(Clone, Debug)]
pub struct ExprIdent {
    pub name: SpannedString,
}
impl ExprIdent {
    pub fn new(name: SpannedString) -> Self {
        Self { name }
    }
    /// get the span of the identifier
    pub fn span(&self) -> Span {
        self.name.span()
    }
}
impl From<Token> for ExprIdent {
    fn from(t: Token) -> Self {
        Self::new(t.into())
    }
}

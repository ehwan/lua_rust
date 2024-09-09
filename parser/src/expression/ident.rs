use crate::Span;
use crate::Token;
use lua_tokenizer::TokenType;

/// just identifier
#[derive(Clone, Debug)]
pub struct ExprIdent {
    pub name: String,
    pub span: Span,
}
impl ExprIdent {
    pub fn new(name: String, span: Span) -> Self {
        Self { name, span }
    }
    /// get the span of the identifier
    pub fn span(&self) -> Span {
        self.span
    }
}
impl From<Token> for ExprIdent {
    fn from(t: Token) -> Self {
        match t.token_type {
            TokenType::Ident(name) => Self::new(name, t.span),
            _ => unreachable!(),
        }
    }
}

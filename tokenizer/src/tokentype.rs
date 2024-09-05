use crate::Literal;
/// classifies the type of token
#[derive(Debug, Clone)]
pub enum TokenType {
    Ident(String),
    Punct(char),
    Literal(Literal),

    // Keywords, this will be converted from `Ident`
    And,
    Break,
    Do,
    Else,
    Elseif,
    End,
    For,
    Function,
    Goto,
    If,
    In,
    Local,
    Not,
    Or,
    Repeat,
    Return,
    Then,
    Until,
    While,
}

impl std::hash::Hash for TokenType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        if let Self::Punct(punct) = self {
            punct.hash(state);
        }
    }
}
impl std::cmp::PartialEq for TokenType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Punct(a), Self::Punct(b)) => a == b,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}
impl std::cmp::Eq for TokenType {}

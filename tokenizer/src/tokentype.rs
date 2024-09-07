use crate::IntOrFloat;

/// classifies the type of token
#[derive(Debug, Clone)]
pub enum TokenType {
    Ident(String),

    Numeric(IntOrFloat),
    String(String),
    Bool(bool),
    Nil,

    // Punctuations
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Caret,
    Hash,
    Ampersand,
    Tilde,
    Pipe,
    LessLess,
    GreaterGreater,
    SlashSlash,
    EqualEqual,
    TildeEqual,
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
    Equal,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    ColonColon,
    Semicolon,
    Colon,
    Comma,
    Dot,
    DotDot,
    DotDotDot,

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

    Eof,
}

impl TokenType {
    pub fn into_ident(self) -> Option<String> {
        match self {
            Self::Ident(ident) => Some(ident),
            _ => None,
        }
    }
    pub fn into_string(self) -> Option<String> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }
    pub fn into_bool(self) -> Option<bool> {
        match self {
            Self::Bool(boolean) => Some(boolean),
            _ => None,
        }
    }
    pub fn into_numeric(self) -> Option<IntOrFloat> {
        match self {
            Self::Numeric(num) => Some(num),
            _ => None,
        }
    }
}

impl std::hash::Hash for TokenType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}
impl std::cmp::PartialEq for TokenType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
impl std::cmp::Eq for TokenType {}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => {
                if ident.is_empty() {
                    write!(f, "<Ident>")
                } else {
                    write!(f, "<{}:Ident>", ident)
                }
            }
            Self::Numeric(num) => {
                write!(f, "{}", num)
            }
            Self::String(string) => {
                write!(f, "\"{}\"", string)
            }
            Self::Bool(boolean) => {
                write!(f, "'{}'", boolean)
            }
            Self::Nil => {
                write!(f, "nil")
            }
            Self::Plus => {
                write!(f, "+")
            }
            Self::Minus => {
                write!(f, "-")
            }
            Self::Asterisk => {
                write!(f, "*")
            }
            Self::Slash => {
                write!(f, "/")
            }
            Self::Percent => {
                write!(f, "%")
            }
            Self::Caret => {
                write!(f, "^")
            }
            Self::Hash => {
                write!(f, "#")
            }
            Self::Ampersand => {
                write!(f, "&")
            }
            Self::Tilde => {
                write!(f, "~")
            }
            Self::Pipe => {
                write!(f, "|")
            }
            Self::LessLess => {
                write!(f, "<<")
            }
            Self::GreaterGreater => {
                write!(f, ">>")
            }
            Self::SlashSlash => {
                write!(f, "//")
            }
            Self::EqualEqual => {
                write!(f, "==")
            }
            Self::TildeEqual => {
                write!(f, "~=")
            }
            Self::LessEqual => {
                write!(f, "<=")
            }
            Self::GreaterEqual => {
                write!(f, ">=")
            }
            Self::Less => {
                write!(f, "<")
            }
            Self::Greater => {
                write!(f, ">")
            }
            Self::Equal => {
                write!(f, "=")
            }
            Self::LParen => {
                write!(f, "(")
            }
            Self::RParen => {
                write!(f, ")")
            }
            Self::LBrace => {
                write!(f, "{{")
            }
            Self::RBrace => {
                write!(f, "}}")
            }
            Self::LBracket => {
                write!(f, "[")
            }
            Self::RBracket => {
                write!(f, "]")
            }
            Self::ColonColon => {
                write!(f, "::")
            }
            Self::Semicolon => {
                write!(f, ";")
            }
            Self::Colon => {
                write!(f, ":")
            }
            Self::Comma => {
                write!(f, ",")
            }
            Self::Dot => {
                write!(f, ".")
            }
            Self::DotDot => {
                write!(f, "..")
            }
            Self::DotDotDot => {
                write!(f, "...")
            }
            Self::And => {
                write!(f, "and")
            }
            Self::Break => {
                write!(f, "break")
            }
            Self::Do => {
                write!(f, "do")
            }
            Self::Else => {
                write!(f, "else")
            }
            Self::Elseif => {
                write!(f, "elseif")
            }
            Self::End => {
                write!(f, "end")
            }
            Self::For => {
                write!(f, "for")
            }
            Self::Function => {
                write!(f, "function")
            }
            Self::Goto => {
                write!(f, "goto")
            }
            Self::If => {
                write!(f, "if")
            }
            Self::In => {
                write!(f, "in")
            }
            Self::Local => {
                write!(f, "local")
            }
            Self::Not => {
                write!(f, "not")
            }
            Self::Or => {
                write!(f, "or")
            }
            Self::Repeat => {
                write!(f, "repeat")
            }
            Self::Return => {
                write!(f, "return")
            }
            Self::Then => {
                write!(f, "then")
            }
            Self::Until => {
                write!(f, "until")
            }
            Self::While => {
                write!(f, "while")
            }
            Self::Eof => {
                write!(f, "<eof>")
            }
        }?;
        write!(f, " ")
    }
}

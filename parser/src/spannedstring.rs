use std::ops::Deref;
use std::ops::DerefMut;

use crate::Span;

/// string with span information.
#[derive(Clone, Debug)]
pub struct SpannedString {
    pub string: String,
    pub span: Span,
}
impl From<crate::Token> for SpannedString {
    fn from(value: crate::Token) -> Self {
        if let lua_tokenizer::TokenType::Ident(val) = value.token_type {
            Self {
                string: val,
                span: value.span,
            }
        } else {
            unreachable!("expecting Ident token, but got {:?}", value);
        }
    }
}
impl Into<String> for SpannedString {
    fn into(self) -> String {
        self.string
    }
}
impl Deref for SpannedString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}
impl DerefMut for SpannedString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.string
    }
}
impl std::fmt::Display for SpannedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}
impl std::hash::Hash for SpannedString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.string.hash(state);
    }
}
impl PartialEq for SpannedString {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
    }
}
impl Eq for SpannedString {}

impl std::cmp::PartialOrd for SpannedString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.string.partial_cmp(&other.string)
    }
}
impl std::cmp::Ord for SpannedString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.string.cmp(&other.string)
    }
}

impl SpannedString {
    pub fn new(string: String, span: Span) -> Self {
        Self { string, span }
    }
    pub fn span(&self) -> Span {
        self.span
    }
}

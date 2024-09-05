use crate::TokenType;

/// Token classification and metadata.
#[derive(Debug, Clone)]
pub struct Token {
    /// token classification
    pub token_type: TokenType,

    /// byte offset of start position
    pub byte_start: usize,
    pub byte_end: usize,
}

impl std::hash::Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.token_type.hash(state);
    }
}
impl std::cmp::PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.token_type == other.token_type
    }
}
impl std::cmp::Eq for Token {}

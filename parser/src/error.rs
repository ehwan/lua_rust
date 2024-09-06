pub use lua_tokenizer::TokenizeError;

#[derive(Debug)]
pub enum ParseError {
    TokenizeError(TokenizeError),
    InvalidToken(InvalidToken),
}

#[derive(Debug)]
pub struct InvalidToken {
    pub byte_start: usize,
    pub byte_end: usize,
}

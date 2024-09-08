use lua_tokenizer::Token;
pub use lua_tokenizer::TokenizeError;

/// error when tokenizing & parsing lua source code
#[non_exhaustive]
#[derive(Debug)]
pub enum ParseError {
    /// error when tokenizing lua source code
    TokenizeError(TokenizeError),
    /// error when feeding token to parser
    InvalidToken(InvalidToken),
    /// when there are multiple possible paths to parse (in GLR parser).
    /// normally, this should not happen
    Ambiguous,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::TokenizeError(e) => write!(f, "{}", e),
            ParseError::InvalidToken(e) => write!(f, "{}", e),
            ParseError::Ambiguous => write!(
                f,
                "Ambiguous Grammar: I guess the source code is not complete"
            ),
        }
    }
}
impl std::error::Error for ParseError
where
    TokenizeError: 'static,
    InvalidToken: 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::TokenizeError(e) => Some(e),
            ParseError::InvalidToken(e) => Some(e),
            ParseError::Ambiguous => None,
        }
    }
}

#[derive(Debug)]
pub struct InvalidToken {
    /// the token that caused the error
    pub token: Option<Token>,
    /// expected tokens
    pub expected: Vec<Token>,
    /// expected nonterminal tokens
    pub expected_nonterm: Vec<parser_expanded::ChunkNonTerminals>,
}
impl std::fmt::Display for InvalidToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(token) = &self.token {
            write!(f, "Invalid token: {:?}", token)?;
        } else {
            write!(f, "Invalid token: EOF")?;
        }
        write!(f, "\nExpected one of: ")?;
        for (i, token) in self.expected.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", token)?;
        }
        write!(f, "\nExpected nonterminals: ")?;
        for (i, token) in self.expected_nonterm.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", token)?;
        }
        Ok(())
    }
}
impl std::error::Error for InvalidToken {}

#[cfg(feature = "diag")]
use codespan_reporting::diagnostic::Diagnostic;
#[cfg(feature = "diag")]
use codespan_reporting::diagnostic::Label;

use crate::parser_expanded;

impl InvalidToken {
    #[cfg(feature = "diag")]
    pub fn to_diag<FileId: Copy>(&self, fileid: FileId) -> Diagnostic<FileId> {
        let mut note_message1 = String::from("Expected one of: ");
        for (idx, token) in self.expected.iter().enumerate() {
            if idx != 0 {
                note_message1.push_str(", ");
            }
            note_message1.push_str(format!("{}", token).as_str());
        }
        let mut note_message2 = String::from("Expected(NonTerminals) one of: ");
        for (idx, token) in self.expected_nonterm.iter().enumerate() {
            if idx != 0 {
                note_message2.push_str(", ");
            }
            note_message2.push_str(format!("{}", token).as_str());
        }
        if let Some(token) = &self.token {
            Diagnostic::error()
                .with_message(format!("Invalid token: {}", token))
                .with_labels(vec![
                    Label::primary(fileid, token.span).with_message("token fed here")
                ])
        } else {
            Diagnostic::error().with_message("code not complete")
        }
        .with_notes(vec![note_message1, note_message2])
    }
}

impl ParseError {
    #[cfg(feature = "diag")]
    pub fn to_diag<FileId: Copy>(&self, fileid: FileId) -> Diagnostic<FileId> {
        match self {
            ParseError::TokenizeError(e) => e.to_diag(fileid),
            ParseError::InvalidToken(e) => e.to_diag(fileid),
            ParseError::Ambiguous => Diagnostic::error()
                .with_message("Ambiguous Grammar: I guess the source code is not complete"),
        }
    }
}

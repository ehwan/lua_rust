use std::fmt::Display;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum TokenizeError {
    ShortStringNewline {
        /// The starting position of the string
        start: usize,
        /// The position of newline
        pos: usize,
    },
    ShortStringNotClosed {
        delim: char,
        /// The starting position of the string
        start: usize,
        /// The position of end of string
        end: usize,
    },
    ShortStringInvalidEscape {
        /// The starting position of the string
        start: usize,
        /// The position of the invalid character
        pos: usize,
        escape: char,
    },
    /// for \xXX escape
    ShortStringNotHex {
        /// The starting position of the string
        start: usize,
        /// The position of the invalid character
        pos: usize,
    },
    /// for \ddd escape
    ShortStringNotDecimal {
        /// The starting position of the string
        start: usize,
        /// The position of the invalid character
        pos: usize,
    },
    /// for \u{XXX} escape
    ShortStringNoOpenBrace {
        /// The starting position of the string
        start: usize,
        /// The position of the invalid character
        pos: usize,
    },
    /// for \u{XXX} escape
    /// codepoint exceeds 2^31
    ShortStringOverflow {
        /// The starting position of the string
        start: usize,
        /// The position of the invalid character
        pos: usize,
    },
    /// for \u{XXX} escape
    ShortStringEmptyCodepoint {
        /// The starting position of the string
        start: usize,

        /// The position of the start of the escape
        escape_start: usize,
        /// The position of the end of the escape (exclusive)
        escape_end: usize,
    },
    LongStringNotClosed {
        /// The starting position of the string
        start: usize,
        end: usize,
        /// number of '=' characters for opening long string
        equal_count: usize,
    },
    InvalidUtf8 {
        /// The start position of the string
        start: usize,
        /// The end position of the string (exclusive)
        end: usize,

        error: FromUtf8Error,
    },

    InvalidPunct {
        /// The position of the invalid character
        pos: usize,
        punct: char,
    },

    MultilineCommentNotClosed {
        /// The starting position of the comment
        start: usize,
        end: usize,
    },

    NumericEmpty {
        /// The starting position of the numeric
        start: usize,
        pos: usize,
    },
}

impl Display for TokenizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenizeError: ")?;
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "diag")]
use codespan_reporting::diagnostic::Diagnostic;
#[cfg(feature = "diag")]
use codespan_reporting::diagnostic::Label;

impl TokenizeError {
    #[cfg(feature = "diag")]
    pub fn to_diag<FileId: Copy>(&self, fileid: FileId) -> Diagnostic<FileId> {
        match self {
            Self::ShortStringNewline { start, pos } => Diagnostic::error()
                .with_message("Newline in short string")
                .with_labels(vec![
                    Label::primary(fileid, *pos..*pos + 1).with_message("newline here"),
                    Label::secondary(fileid, *start..*pos).with_message("string definition here"),
                ])
                .with_notes(vec![
                    "Use escaped newline \\n or put backslash right before newline.".to_string(),
                ]),
            Self::ShortStringNotClosed { delim, start, end } => Diagnostic::error()
                .with_message("Short string not closed")
                .with_labels(vec![
                    Label::primary(fileid, *start..*end).with_message("string definition here")
                ])
                .with_notes(vec![format!("Expected closing delimiter '{}'", delim)]),
            Self::ShortStringInvalidEscape { start, pos, escape } => Diagnostic::error()
                .with_message("Invalid escape sequence")
                .with_labels(vec![
                    Label::primary(fileid, *pos..*pos + 1).with_message("invalid escape sequence"),
                    Label::secondary(fileid, *start..*pos).with_message("string definition here"),
                ])
                .with_notes(vec![format!("Invalid escape sequence: {}", escape)]),
            Self::ShortStringNotHex { start, pos } => Diagnostic::error()
                .with_message("Invalid hex escape sequence")
                .with_labels(vec![
                    Label::primary(fileid, *pos..*pos + 1).with_message("expected hex digit here"),
                    Label::secondary(fileid, *start..*pos).with_message("String definition here"),
                ])
                .with_notes(vec![
                    "Hex escape sequence should be in the form of \\xHH".to_string()
                ]),
            Self::ShortStringNotDecimal { start, pos } => Diagnostic::error()
                .with_message("Invalid digit escape sequence")
                .with_labels(vec![
                    Label::primary(fileid, *pos..*pos + 1).with_message("expected digit here"),
                    Label::secondary(fileid, *start..*pos).with_message("String definition here"),
                ])
                .with_notes(vec![
                    "digit escape sequence should be in the form of \\ddd".to_string()
                ]),
            Self::ShortStringNoOpenBrace { start, pos } => Diagnostic::error()
                .with_message("Open brace expected")
                .with_labels(vec![
                    Label::primary(fileid, *pos..*pos + 1).with_message("expected '{' here"),
                    Label::secondary(fileid, *start..*pos).with_message("String definition here"),
                ])
                .with_notes(vec![
                    "unicode escape sequence should be in the form of \\u{X+}".to_string(),
                ]),
            Self::ShortStringOverflow { start, pos } => Diagnostic::error()
                .with_message("Unicode codepoint overflowed")
                .with_labels(vec![
                    Label::primary(fileid, *pos..*pos + 1).with_message("overflowed here"),
                    Label::secondary(fileid, *start..*pos).with_message("String definition here"),
                ])
                .with_notes(vec!["unicode codepoint must be below 2^31".to_string()]),

            Self::ShortStringEmptyCodepoint {
                start,
                escape_start,
                escape_end,
            } => Diagnostic::error()
                .with_message("Empty unicode codepoint")
                .with_labels(vec![
                    Label::primary(fileid, *escape_start..*escape_end)
                        .with_message("escape sequence here"),
                    Label::secondary(fileid, *start..*escape_end)
                        .with_message("String definition here"),
                ])
                .with_notes(vec![
                    "unicode escape sequence should be in the form of \\u{X+}".to_string(),
                ]),
            Self::LongStringNotClosed {
                start,
                end,
                equal_count,
            } => Diagnostic::error()
                .with_message("long string literal not closed")
                .with_labels(vec![
                    Label::primary(fileid, *start..*end).with_message("string definition here")
                ])
                .with_notes(vec![
                    "must close with the same number of '=' characters as opening".to_string(),
                    format!("]{}]", "=".repeat(*equal_count)),
                ]),

            Self::InvalidUtf8 { start, end, error } => Diagnostic::error()
                .with_message("Invalid UTF-8 string")
                .with_labels(vec![
                    Label::primary(fileid, *start..*end).with_message("string definition here")
                ])
                .with_notes(vec![error.to_string()]),

            Self::InvalidPunct { pos, punct } => Diagnostic::error()
                .with_message(format!("Invalid punctuation: '{}'", punct))
                .with_labels(vec![
                    Label::primary(fileid, *pos..*pos + 1).with_message("punctuation here")
                ]),

            Self::MultilineCommentNotClosed { start, end } => Diagnostic::error()
                .with_message("Multiline comment not closed")
                .with_labels(vec![
                    Label::primary(fileid, *start..*end).with_message("comment definition here")
                ]),

            Self::NumericEmpty { start, pos } => Diagnostic::error()
                .with_message("Empty numeric literal")
                .with_labels(vec![
                    Label::primary(fileid, *pos..*pos + 1).with_message("empty here"),
                    Label::secondary(fileid, *start..*pos).with_message("numeric definition here"),
                ]),
        }
    }
}

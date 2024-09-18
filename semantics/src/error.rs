use std::fmt::Display;

use lua_parser::{Span, SpannedString};

#[non_exhaustive]
#[derive(Debug)]
pub enum ProcessError {
    MultipleLabel(SpannedString),
    VariadicOutsideFunction(Span),
    VariadicInNonVariadicFunction(Span),
    BreakOutsideLoop(Span),
    InvalidGotoScope(Span, Span),
    InvalidLabel(Span),
}

impl Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::MultipleLabel(s) => write!(f, "Multiple label: {}", s.string),
            ProcessError::VariadicOutsideFunction(_) => {
                write!(f, "Variadic outside function")
            }
            ProcessError::VariadicInNonVariadicFunction(_) => {
                write!(f, "Variadic in non-variadic function")
            }
            ProcessError::BreakOutsideLoop(_) => write!(f, "Break outside loop"),
            ProcessError::InvalidGotoScope(_, _) => {
                write!(f, "Invalid goto")
            }
            ProcessError::InvalidLabel(_) => write!(f, "Invalid label"),
        }
    }
}
impl std::error::Error for ProcessError {}

#[cfg(feature = "diag")]
use codespan_reporting::diagnostic::Diagnostic;
#[cfg(feature = "diag")]
use codespan_reporting::diagnostic::Label;

impl ProcessError {
    #[cfg(feature = "diag")]
    pub fn to_diag<FileId: Copy>(&self, fileid: FileId) -> Diagnostic<FileId> {
        let message = self.to_string();
        match self {
            ProcessError::MultipleLabel(s) => Diagnostic::error()
                .with_message(message)
                .with_labels(vec![Label::primary(fileid, s.span)]),
            ProcessError::VariadicOutsideFunction(span) => Diagnostic::error()
                .with_message(message)
                .with_labels(vec![Label::primary(fileid, *span)]),
            ProcessError::VariadicInNonVariadicFunction(span) => Diagnostic::error()
                .with_message(message)
                .with_labels(vec![Label::primary(fileid, *span)]),
            ProcessError::BreakOutsideLoop(span) => Diagnostic::error()
                .with_message(message)
                .with_labels(vec![Label::primary(fileid, *span)]),
            ProcessError::InvalidGotoScope(label_span, goto_span) => {
                Diagnostic::error().with_message(message).with_labels(vec![
                    Label::primary(fileid, *goto_span).with_message("goto here"),
                    Label::secondary(fileid, *label_span).with_message("label defined here"),
                ])
            }
            ProcessError::InvalidLabel(span) => Diagnostic::error()
                .with_message(message)
                .with_labels(vec![Label::primary(fileid, *span)]),
        }
    }
}

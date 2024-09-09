use super::Expression;
use crate::Span;

/// unary operation. `OP x`
#[derive(Clone, Debug)]
pub enum ExprUnary {
    /// `-x`
    Minus(ExprUnaryData),
    /// `+x`, no-op, but compile error if x is not a number
    Plus(ExprUnaryData),
    /// `~x`
    BitwiseNot(ExprUnaryData),
    /// `#x`
    Length(ExprUnaryData),
    /// `not x`
    LogicalNot(ExprUnaryData),
}

impl ExprUnary {
    /// get the span of the whole binary expression
    pub fn span(&self) -> Span {
        match self {
            Self::Minus(value) => value.span(),
            Self::Plus(value) => value.span(),
            Self::BitwiseNot(value) => value.span(),
            Self::Length(value) => value.span(),
            Self::LogicalNot(value) => value.span(),
        }
    }
    /// get the span of the operator
    pub fn span_op(&self) -> Span {
        match self {
            Self::Minus(value) => value.span_op(),
            Self::Plus(value) => value.span_op(),
            Self::BitwiseNot(value) => value.span_op(),
            Self::Length(value) => value.span_op(),
            Self::LogicalNot(value) => value.span_op(),
        }
    }
}

/// Internal data for unary operation
#[derive(Clone, Debug)]
pub struct ExprUnaryData {
    pub value: Box<Expression>,

    /// span that covers the whole binary expression
    pub span: Span,
    /// span of the operator
    pub span_op: Span,
}
impl ExprUnaryData {
    pub fn new(value: Expression, span: Span, span_op: Span) -> Self {
        Self {
            value: Box::new(value),
            span,
            span_op,
        }
    }
    /// get the span of the whole unary expression
    pub fn span(&self) -> Span {
        self.span
    }
    /// get the span of the operator
    pub fn span_op(&self) -> Span {
        self.span_op
    }
}

use super::Expression;
use super::Span;

/// binary operation. `lhs OP rhs`
#[derive(Clone, Debug)]
pub enum ExprBinary {
    /// `lhs + rhs`
    Add(ExprBinaryData),
    /// `lhs - rhs`
    Sub(ExprBinaryData),
    /// `lhs * rhs`
    Mul(ExprBinaryData),
    /// `lhs / rhs`: float division
    Div(ExprBinaryData),

    /// `lhs // rhs`: floor division
    FloorDiv(ExprBinaryData),
    /// `lhs % rhs`
    Mod(ExprBinaryData),
    /// `lhs ^ rhs`, right associative
    Pow(ExprBinaryData),

    /// `lhs .. rhs`, right associative
    Concat(ExprBinaryData),

    /// `lhs & rhs`
    BitwiseAnd(ExprBinaryData),
    /// `lhs | rhs`
    BitwiseOr(ExprBinaryData),
    /// `lhs ~ rhs`
    BitwiseXor(ExprBinaryData),
    /// `lhs << rhs`
    ShiftLeft(ExprBinaryData),
    /// `lhs >> rhs`
    ShiftRight(ExprBinaryData),

    /// `lhs == rhs`
    Equal(ExprBinaryData),
    /// `lhs ~= rhs`
    NotEqual(ExprBinaryData),
    /// `lhs < rhs`
    LessThan(ExprBinaryData),
    /// `lhs <= rhs`
    LessEqual(ExprBinaryData),
    /// `lhs > rhs`
    GreaterThan(ExprBinaryData),
    /// `lhs >= rhs`
    GreaterEqual(ExprBinaryData),

    /// `lhs and rhs`
    LogicalAnd(ExprBinaryData),

    /// `lhs or rhs`
    LogicalOr(ExprBinaryData),
}
impl ExprBinary {
    /// get the span of the whole binary expression
    pub fn span(&self) -> Span {
        match self {
            Self::Add(data) => data.span(),
            Self::Sub(data) => data.span(),
            Self::Mul(data) => data.span(),
            Self::Div(data) => data.span(),
            Self::FloorDiv(data) => data.span(),
            Self::Mod(data) => data.span(),
            Self::Pow(data) => data.span(),
            Self::Concat(data) => data.span(),
            Self::BitwiseAnd(data) => data.span(),
            Self::BitwiseOr(data) => data.span(),
            Self::BitwiseXor(data) => data.span(),
            Self::ShiftLeft(data) => data.span(),
            Self::ShiftRight(data) => data.span(),
            Self::Equal(data) => data.span(),
            Self::NotEqual(data) => data.span(),
            Self::LessThan(data) => data.span(),
            Self::LessEqual(data) => data.span(),
            Self::GreaterThan(data) => data.span(),
            Self::GreaterEqual(data) => data.span(),
            Self::LogicalAnd(data) => data.span(),
            Self::LogicalOr(data) => data.span(),
        }
    }
    /// get the span of the operator
    pub fn span_op(&self) -> Span {
        match self {
            Self::Add(data) => data.span_op(),
            Self::Sub(data) => data.span_op(),
            Self::Mul(data) => data.span_op(),
            Self::Div(data) => data.span_op(),
            Self::FloorDiv(data) => data.span_op(),
            Self::Mod(data) => data.span_op(),
            Self::Pow(data) => data.span_op(),
            Self::Concat(data) => data.span_op(),
            Self::BitwiseAnd(data) => data.span_op(),
            Self::BitwiseOr(data) => data.span_op(),
            Self::BitwiseXor(data) => data.span_op(),
            Self::ShiftLeft(data) => data.span_op(),
            Self::ShiftRight(data) => data.span_op(),
            Self::Equal(data) => data.span_op(),
            Self::NotEqual(data) => data.span_op(),
            Self::LessThan(data) => data.span_op(),
            Self::LessEqual(data) => data.span_op(),
            Self::GreaterThan(data) => data.span_op(),
            Self::GreaterEqual(data) => data.span_op(),
            Self::LogicalAnd(data) => data.span_op(),
            Self::LogicalOr(data) => data.span_op(),
        }
    }
}

/// Internal data for binary operation
#[derive(Clone, Debug)]
pub struct ExprBinaryData {
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,

    /// span that covers the whole binary expression
    pub span: Span,
    /// span of the operator
    pub span_op: Span,
}
impl ExprBinaryData {
    pub fn new(lhs: Expression, rhs: Expression, span: Span, span_op: Span) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
            span_op,
        }
    }
    /// get the span of the whole binary expression
    pub fn span(&self) -> Span {
        self.span
    }
    /// get the span of the operator
    pub fn span_op(&self) -> Span {
        self.span_op
    }
}

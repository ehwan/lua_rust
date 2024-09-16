use super::Expression;

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
impl ExprBinary {}

/// Internal data for binary operation
#[derive(Clone, Debug)]
pub struct ExprBinaryData {
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}
impl ExprBinaryData {
    pub fn new(lhs: Expression, rhs: Expression) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }
}

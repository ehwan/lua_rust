use super::Expression;

/// binary operation. `lhs OP rhs`
#[derive(Clone, Debug)]
pub enum ExprBinary {
    /// `lhs + rhs`
    Add(Box<Expression>, Box<Expression>),
    /// `lhs - rhs`
    Sub(Box<Expression>, Box<Expression>),
    /// `lhs * rhs`
    Mul(Box<Expression>, Box<Expression>),
    /// `lhs / rhs`: float division
    Div(Box<Expression>, Box<Expression>),

    /// `lhs // rhs`: floor division
    FloorDiv(Box<Expression>, Box<Expression>),
    /// `lhs % rhs`
    Mod(Box<Expression>, Box<Expression>),
    /// `lhs ^ rhs`, right associative
    Pow(Box<Expression>, Box<Expression>),

    /// `lhs .. rhs`, right associative
    Concat(Box<Expression>, Box<Expression>),

    /// `lhs & rhs`
    BitwiseAnd(Box<Expression>, Box<Expression>),
    /// `lhs | rhs`
    BitwiseOr(Box<Expression>, Box<Expression>),
    /// `lhs ~ rhs`
    BitwiseXor(Box<Expression>, Box<Expression>),
    /// `lhs << rhs`
    ShiftLeft(Box<Expression>, Box<Expression>),
    /// `lhs >> rhs`
    ShiftRight(Box<Expression>, Box<Expression>),

    /// `lhs == rhs`
    Equal(Box<Expression>, Box<Expression>),
    /// `lhs ~= rhs`
    NotEqual(Box<Expression>, Box<Expression>),
    /// `lhs < rhs`
    LessThan(Box<Expression>, Box<Expression>),
    /// `lhs <= rhs`
    LessEqual(Box<Expression>, Box<Expression>),
    /// `lhs > rhs`
    GreaterThan(Box<Expression>, Box<Expression>),
    /// `lhs >= rhs`
    GreaterEqual(Box<Expression>, Box<Expression>),

    /// `lhs and rhs`
    LogicalAnd(Box<Expression>, Box<Expression>),

    /// `lhs or rhs`
    LogicalOr(Box<Expression>, Box<Expression>),
}

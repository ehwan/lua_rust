use super::Expression;

/// unary operation. `OP x`
#[derive(Clone, Debug)]
pub enum ExprUnary {
    /// `-x`
    Minus(Box<Expression>),
    /// `~x`
    BitwiseNot(Box<Expression>),
    /// `#x`
    Length(Box<Expression>),
    /// `not x`
    LogicalNot(Box<Expression>),
}

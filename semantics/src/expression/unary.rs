use super::Expression;

/// unary operation. `OP x`
#[derive(Clone, Debug)]
pub enum ExprUnary {
    /// `-x`
    Minus(ExprUnaryData),
    /// `~x`
    BitwiseNot(ExprUnaryData),
    /// `#x`
    Length(ExprUnaryData),
    /// `not x`
    LogicalNot(ExprUnaryData),
}

impl ExprUnary {}

/// Internal data for unary operation
#[derive(Clone, Debug)]
pub struct ExprUnaryData {
    pub value: Box<Expression>,
}
impl ExprUnaryData {
    pub fn new(value: Expression) -> Self {
        Self {
            value: Box::new(value),
        }
    }
}

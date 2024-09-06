use crate::Expression;

/// function call statement.
#[derive(Clone, Debug)]
pub struct StmtFunctionCall {
    pub prefix: Expression,
    pub args: Vec<Expression>,
}
impl StmtFunctionCall {
    pub fn new(prefix: Expression, args: Vec<Expression>) -> Self {
        Self { prefix, args }
    }
}

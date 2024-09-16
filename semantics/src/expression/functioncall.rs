use super::Expression;

/// function call. `prefix(args)` or `prefix:method(args)`.
#[derive(Clone, Debug)]
pub struct ExprFunctionCall {
    pub prefix: Box<Expression>,
    pub method: Option<String>,
    pub args: Vec<Expression>,
}
impl ExprFunctionCall {
    pub fn new(prefix: Expression, method: Option<String>, args: Vec<Expression>) -> Self {
        Self {
            prefix: Box::new(prefix),
            method,
            args,
        }
    }
}

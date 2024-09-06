use super::Expression;

/// function call. `prefix(args)` or `prefix:method(args)`.
/// `prefix:method(args)` will be converted to `prefix.method(prefix, args)`.
#[derive(Clone, Debug)]
pub struct ExprFunctionCall {
    pub prefix: Box<Expression>,
    pub args: Vec<Expression>,
}
impl ExprFunctionCall {
    pub fn new(prefix: Expression, args: Vec<Expression>) -> Self {
        Self {
            prefix: Box::new(prefix),
            args,
        }
    }
}

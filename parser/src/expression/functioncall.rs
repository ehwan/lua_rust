use super::Expression;
use crate::{Span, SpannedString};

#[derive(Clone, Debug)]
pub struct FunctionCallArguments {
    /// arguments definition, excluding `self` argument.
    pub args: Vec<Expression>,
    /// span of the whole parameters
    pub span: Span,
}
impl FunctionCallArguments {
    pub fn new(args: Vec<Expression>, span: Span) -> Self {
        Self { args, span }
    }
    /// get the span of the whole parameters
    pub fn span(&self) -> Span {
        self.span
    }
}

/// function call. `prefix(args)` or `prefix:method(args)`.
#[derive(Clone, Debug)]
pub struct ExprFunctionCall {
    pub prefix: Box<Expression>,
    pub method: Option<SpannedString>,
    pub args: FunctionCallArguments,
    pub span: Span,
}
impl ExprFunctionCall {
    pub fn new(
        prefix: Expression,
        method: Option<SpannedString>,
        args: FunctionCallArguments,
        span: Span,
    ) -> Self {
        Self {
            prefix: Box::new(prefix),
            method,
            args,
            span,
        }
    }
    /// get the span of the whole function call
    pub fn span(&self) -> Span {
        self.span
    }
}

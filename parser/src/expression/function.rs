use crate::statement;
use crate::Span;

use super::ExprString;

/// parameter list for named & anonymous function definition
#[derive(Clone, Debug)]
pub struct ParameterList {
    pub names: Vec<ExprString>,
    /// is `...` present?
    pub variadic: bool,

    pub span: Span,
}
impl ParameterList {
    pub fn new(names: Vec<ExprString>, variadic: bool, span: Span) -> Self {
        Self {
            names,
            variadic,
            span,
        }
    }
    /// get the span of the whole parameter list
    pub fn span(&self) -> Span {
        self.span
    }
}

/// unnamed function
#[derive(Clone, Debug)]
pub struct ExprFunction {
    /// function parameters
    pub parameters: ParameterList,
    /// function body to be executed
    pub block: statement::Block,

    /// span of the whole function definition
    pub span: Span,
}
impl ExprFunction {
    pub fn new(parameters: Option<ParameterList>, block: statement::Block, span: Span) -> Self {
        if let Some(p) = parameters {
            Self {
                parameters: p,
                block,
                span,
            }
        } else {
            Self {
                parameters: ParameterList::new(Vec::new(), false, Span::new_none()),
                block,
                span,
            }
        }
    }

    /// get the span of the whole function definition
    pub fn span(&self) -> Span {
        self.span
    }
}

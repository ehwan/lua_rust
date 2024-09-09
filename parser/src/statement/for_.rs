use super::Block;
use crate::Expression;
use crate::{Span, SpannedString};

/// for statement with start, end, step.
#[derive(Clone, Debug)]
pub struct StmtFor {
    pub name: String,
    pub start: Expression,
    pub end: Expression,
    pub step: Expression,
    pub block: Block,
    pub span: Span,
}
impl StmtFor {
    pub fn new(
        name: String,
        start: Expression,
        end: Expression,
        step: Expression,
        block: Block,
        span: Span,
    ) -> Self {
        Self {
            name,
            start,
            end,
            step,
            block,
            span,
        }
    }
    /// get the span of the whole for statement
    pub fn span(&self) -> Span {
        self.span
    }
}

/// for statement with generic expressions.
#[derive(Clone, Debug)]
pub struct StmtForGeneric {
    pub names: Vec<SpannedString>,
    pub expressions: Vec<Expression>,
    pub block: Block,
    pub span: Span,
}
impl StmtForGeneric {
    pub fn new(
        names: Vec<SpannedString>,
        expressions: Vec<Expression>,
        block: Block,
        span: Span,
    ) -> Self {
        Self {
            names,
            expressions,
            block,
            span,
        }
    }
    /// get the span of the whole for statement
    pub fn span(&self) -> Span {
        self.span
    }
}

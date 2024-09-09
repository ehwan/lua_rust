use super::Expression;
use crate::Span;

/// `table[index]`, `table.index`
#[derive(Clone, Debug)]
pub struct ExprTableIndex {
    pub table: Box<Expression>,
    pub index: Box<Expression>,
    pub span: Span,
}
impl ExprTableIndex {
    pub fn new(table: Expression, index: Expression, span: Span) -> Self {
        Self {
            table: Box::new(table),
            index: Box::new(index),
            span,
        }
    }
    /// get the span of the whole table index expression
    pub fn span(&self) -> Span {
        self.span
    }
}

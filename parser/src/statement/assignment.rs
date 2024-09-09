use crate::Expression;
use crate::Span;

/// `l0, l1, l2 = r0, r1, r2`.
/// variadic `...` can be used in both `l` and `r`
#[derive(Clone, Debug)]
pub struct StmtAssignment {
    pub lhs: Vec<Expression>,
    pub rhs: Vec<Expression>,

    /// span of the whole assignment statement
    pub span: Span,
    /// span of the `=`
    pub span_op: Span,
}
impl StmtAssignment {
    pub fn new(lhs: Vec<Expression>, rhs: Vec<Expression>, span: Span, span_op: Span) -> Self {
        // @TODO check variadic here
        Self {
            lhs,
            rhs,
            span,
            span_op,
        }
    }
    /// get the span of the whole assignment statement
    pub fn span(&self) -> Span {
        self.span
    }
    /// get the span of the `=`
    pub fn span_op(&self) -> Span {
        self.span_op
    }
}

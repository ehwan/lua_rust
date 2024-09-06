use crate::Expression;

/// `l0, l1, l2 = r0, r1, r2`.
/// variadic `...` can be used in both `l` and `r`
#[derive(Clone, Debug)]
pub struct StmtAssignment {
    pub lhs: Vec<Expression>,
    pub rhs: Vec<Expression>,
}
impl StmtAssignment {
    pub fn new(lhs: Vec<Expression>, rhs: Vec<Expression>) -> Self {
        // @TODO check variadic here
        Self { lhs, rhs }
    }
}

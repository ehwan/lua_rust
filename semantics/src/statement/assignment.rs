use crate::Expression;

/// `l0, l1, l2 = r0, r1, r2`.
/// variadic `...` cannot be used in `lhs`
#[derive(Clone, Debug)]
pub struct StmtAssignment {
    pub lhs: Vec<Expression>,
    pub rhs: Vec<Expression>,
}
impl StmtAssignment {
    pub fn new(lhs: Vec<Expression>, rhs: Vec<Expression>) -> Self {
        Self { lhs, rhs }
    }
}

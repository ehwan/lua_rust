use super::Expression;

/// `table[index]`, `table.index`
#[derive(Clone, Debug)]
pub struct ExprTableIndex {
    pub table: Box<Expression>,
    pub index: Box<Expression>,
}
impl ExprTableIndex {
    pub fn new(table: Expression, index: Expression) -> Self {
        Self {
            table: Box::new(table),
            index: Box::new(index),
        }
    }
}

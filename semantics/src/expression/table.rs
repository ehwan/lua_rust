use super::Expression;
use crate::IntType;

/// table constructor, a list of fields
#[derive(Clone, Debug)]
pub struct ExprTableConstructor {
    /// (key,value) pair fields
    pub fields: Vec<(Expression, Expression)>,
    /// if last element of given table constructor is just value(without key), we must check if it is Multire.
    pub last_value_field: Option<(IntType, Box<Expression>)>,
}
impl ExprTableConstructor {
    pub fn new(
        fields: Vec<(Expression, Expression)>,
        last_value_field: Option<(IntType, Box<Expression>)>,
    ) -> Self {
        Self {
            fields,
            last_value_field,
        }
    }
}

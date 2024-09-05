use crate::FloatType;
use crate::IntType;

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(IntType),
    Floating(FloatType),
    String(String),
    Bool(bool),
    Nil,
}

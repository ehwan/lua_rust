use super::Expression;
use super::FloatType;
use super::IntOrFloat;
use super::IntType;

impl From<String> for Expression {
    fn from(s: String) -> Self {
        Expression::String(ExprString::new(s))
    }
}
impl From<IntOrFloat> for Expression {
    fn from(x: IntOrFloat) -> Self {
        Expression::Numeric(ExprNumeric::new(x))
    }
}
impl From<IntType> for Expression {
    fn from(x: IntType) -> Self {
        Expression::Numeric(ExprNumeric::new(IntOrFloat::Int(x)))
    }
}
impl From<FloatType> for Expression {
    fn from(x: FloatType) -> Self {
        Expression::Numeric(ExprNumeric::new(IntOrFloat::Float(x)))
    }
}
impl From<bool> for Expression {
    fn from(x: bool) -> Self {
        Expression::Bool(ExprBool { value: x })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ExprNil;

/// lua numeric literal value
#[derive(Clone, Copy, Debug)]
pub struct ExprNumeric {
    pub value: IntOrFloat,
}
impl ExprNumeric {
    pub fn new(value: IntOrFloat) -> Self {
        Self { value }
    }
}

/// lua string literal value
#[derive(Clone, Debug)]
pub struct ExprString {
    pub value: String,
}
impl ExprString {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

/// lua boolean literal value
#[derive(Clone, Copy, Debug)]
pub struct ExprBool {
    pub value: bool,
}

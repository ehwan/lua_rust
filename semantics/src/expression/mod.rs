use crate::FloatType;
use crate::IntOrFloat;
use crate::IntType;

/// lua value & expression
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Expression {
    /// _ENV
    Env,

    Variadic,

    Nil,
    Boolean(bool),
    Numeric(IntOrFloat),
    String(Vec<u8>),

    /// load from stack
    LocalVariable(ExprLocalVariable),

    /// `table[index]`.
    TableIndex(ExprTableIndex),

    /// binary operation. `lhs OP rhs`
    Binary(ExprBinary),
    /// unary operation. `OP x`
    Unary(ExprUnary),

    /// table constructor
    TableConstructor(ExprTableConstructor),

    /// function call.
    /// `prefix(args)` or `prefix:method(args)`
    FunctionCall(ExprFunctionCall),

    /// function object constructor
    FunctionObject(ExprFunctionObject),
}

impl From<()> for Expression {
    fn from(_: ()) -> Self {
        Expression::Nil
    }
}
impl From<bool> for Expression {
    fn from(value: bool) -> Self {
        Expression::Boolean(value)
    }
}
impl From<IntOrFloat> for Expression {
    fn from(value: IntOrFloat) -> Self {
        Expression::Numeric(value)
    }
}
impl From<IntType> for Expression {
    fn from(value: IntType) -> Self {
        Expression::Numeric(value.into())
    }
}
impl From<FloatType> for Expression {
    fn from(value: FloatType) -> Self {
        Expression::Numeric(value.into())
    }
}
impl From<Vec<u8>> for Expression {
    fn from(value: Vec<u8>) -> Self {
        Expression::String(value)
    }
}
impl From<String> for Expression {
    fn from(value: String) -> Self {
        Expression::String(value.into_bytes())
    }
}

impl Expression {}

// mod literal;
// pub use literal::ExprVariadic;

mod table;
pub use table::ExprTableConstructor;

mod localvar;
pub use localvar::ExprLocalVariable;

mod index;
pub use index::ExprTableIndex;

mod binary;
pub use binary::ExprBinary;
pub use binary::ExprBinaryData;

mod unary;
pub use unary::ExprUnary;
pub use unary::ExprUnaryData;

mod functioncall;
pub use functioncall::ExprFunctionCall;

mod function;
pub use function::ExprFunctionObject;
pub use function::FunctionDefinition;

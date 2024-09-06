pub use lua_tokenizer::IntOrFloat;

use crate::FloatType;
use crate::IntType;

/// lua value & expression
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Expression {
    Nil(ExprNil),
    /// lua numeric literal value
    Numeric(ExprNumeric),
    /// lua string literal value
    String(ExprString),
    /// lua boolean literal value
    Bool(ExprBool),
    /// table constructor
    Table(ExprTable),
    /// unnamed function object
    Function(ExprFunction),

    /// `...`
    Variadic,

    /// function call.
    /// `prefix(args)` or `prefix:method(args)`
    FunctionCall(ExprFunctionCall),

    /// just identifier
    Ident(ExprIdent),
    /// `table[index]`.
    /// `table.index` also matches here.
    TableIndex(ExprTableIndex),
    /// binary operation. `lhs OP rhs`
    Binary(ExprBinary),
    /// unary operation. `OP x`
    Unary(ExprUnary),
}

impl Expression {
    pub fn new_ident(name: String) -> Self {
        Expression::Ident(ExprIdent::new(name))
    }
}

mod literal;
pub use literal::ExprBool;
pub use literal::ExprNil;
pub use literal::ExprNumeric;
pub use literal::ExprString;

mod table;
pub use table::ExprTable;
pub(crate) use table::TableConstructorFieldBuilder;
pub use table::TableField;

mod ident;
pub use ident::ExprIdent;

mod index;
pub use index::ExprTableIndex;

mod binary;
pub use binary::ExprBinary;

mod unary;
pub use unary::ExprUnary;

mod function;
pub use function::ExprFunction;
pub use function::ParameterList;

mod functioncall;
pub use functioncall::ExprFunctionCall;

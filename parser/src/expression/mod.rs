use crate::Span;

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
    /// `...`
    Variadic(ExprVariadic),

    /// just identifier
    Ident(ExprIdent),

    /// table constructor
    Table(ExprTable),
    /// unnamed function object
    Function(ExprFunction),

    /// function call.
    /// `prefix(args)` or `prefix:method(args)`
    FunctionCall(ExprFunctionCall),

    /// `table[index]`.
    /// `table.index` also matches here.
    TableIndex(ExprTableIndex),
    /// binary operation. `lhs OP rhs`
    Binary(ExprBinary),
    /// unary operation. `OP x`
    Unary(ExprUnary),
}

impl Expression {
    /// get the span of the expression
    pub fn span(&self) -> Span {
        match self {
            Self::Nil(v) => v.span(),
            Self::Numeric(v) => v.span(),
            Self::String(v) => v.span(),
            Self::Bool(v) => v.span(),
            Self::Variadic(v) => v.span(),
            Self::Ident(v) => v.span(),
            Self::Table(v) => v.span(),
            Self::Function(v) => v.span(),
            Self::FunctionCall(v) => v.span(),
            Self::TableIndex(v) => v.span(),
            Self::Binary(v) => v.span(),
            Self::Unary(v) => v.span(),
        }
    }
}

mod literal;
pub use literal::ExprBool;
pub use literal::ExprNil;
pub use literal::ExprNumeric;
pub use literal::ExprString;
pub use literal::ExprVariadic;

mod table;
pub use table::ExprTable;
pub use table::TableField;
pub use table::TableFieldKeyValue;
pub use table::TableFieldNameValue;
pub use table::TableFieldValue;

mod ident;
pub use ident::ExprIdent;

mod index;
pub use index::ExprTableIndex;

mod binary;
pub use binary::ExprBinary;
pub use binary::ExprBinaryData;

mod unary;
pub use unary::ExprUnary;
pub use unary::ExprUnaryData;

mod function;
pub use function::ExprFunction;
pub use function::ParameterList;

mod functioncall;
pub use functioncall::ExprFunctionCall;
pub use functioncall::FunctionCallArguments;

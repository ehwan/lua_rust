use lua_tokenizer::{IntOrFloat, IntType};
use std::boxed::Box;

use crate::statement;

/// lua value & expression
#[derive(Clone, Debug)]
pub enum Expression {
    Nil,
    Numeric(Numeric),
    String(StringLiteral),
    Bool(Bool),
    TableConstructor(TableConstructor),
    /// The function object
    FunctionDef(FunctionBody),

    /// '...'
    Variadic,

    FunctionCall(FunctionCall),

    /// just identifier
    Ident(Ident),
    /// `table[index]`
    TableIndex(TableIndex),
    /// binary operation. `lhs op rhs`
    Binary(Binary),
    /// unary operation. `op x`
    Unary(Unary),
}

#[derive(Clone, Copy, Debug)]
pub struct Numeric {
    pub value: IntOrFloat,
}

#[derive(Clone, Debug)]
pub struct StringLiteral {
    pub value: String,
}
impl StringLiteral {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bool {
    pub value: bool,
}

#[derive(Clone, Debug)]
pub enum TableConstructorField {
    KeyValue(Expression, Expression),
    NameValue(String, Expression),
    Value(Expression),
}

#[derive(Clone, Debug)]
pub struct TableConstructor {
    pub fields: Vec<(Expression, Expression)>,
    /// current number of consecutive array elements
    pub(crate) consecutive: IntType,
}

impl TableConstructor {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            consecutive: 0,
        }
    }

    pub fn insert(&mut self, field: TableConstructorField) {
        match field {
            TableConstructorField::KeyValue(key, value) => {
                self.fields.push((key, value));
            }
            TableConstructorField::NameValue(name, value) => {
                let key = Expression::String(StringLiteral { value: name });
                self.fields.push((key, value));
            }
            TableConstructorField::Value(value) => {
                // initial value is 1
                self.consecutive += 1;
                let idx = self.consecutive;
                let key = Expression::Numeric(Numeric {
                    value: IntOrFloat::Int(idx),
                });
                self.fields.push((key, value));
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ident {
    pub name: String,
}

/// `table[index]`
#[derive(Clone, Debug)]
pub struct TableIndex {
    pub table: Box<Expression>,
    pub index: Box<Expression>,
}
impl TableIndex {
    pub fn new(table: Expression, index: Expression) -> Self {
        Self {
            table: Box::new(table),
            index: Box::new(index),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Binary {
    /// `lhs + rhs`
    Add(Box<Expression>, Box<Expression>),
    /// `lhs - rhs`
    Sub(Box<Expression>, Box<Expression>),
    /// `lhs * rhs`
    Mul(Box<Expression>, Box<Expression>),
    /// `lhs / rhs`: float division
    Div(Box<Expression>, Box<Expression>),

    /// `lhs // rhs`: floor division
    FloorDiv(Box<Expression>, Box<Expression>),
    /// `lhs % rhs`
    Mod(Box<Expression>, Box<Expression>),
    /// `lhs ^ rhs`, right associative
    Pow(Box<Expression>, Box<Expression>),

    /// `lhs .. rhs`, right associative
    Concat(Box<Expression>, Box<Expression>),

    /// `lhs & rhs`
    BitwiseAnd(Box<Expression>, Box<Expression>),
    /// `lhs | rhs`
    BitwiseOr(Box<Expression>, Box<Expression>),
    /// `lhs ~ rhs`
    BitwiseXor(Box<Expression>, Box<Expression>),
    /// `lhs << rhs`
    ShiftLeft(Box<Expression>, Box<Expression>),
    /// `lhs >> rhs`
    ShiftRight(Box<Expression>, Box<Expression>),

    /// `lhs == rhs`
    Equal(Box<Expression>, Box<Expression>),
    /// `lhs ~= rhs`
    NotEqual(Box<Expression>, Box<Expression>),
    /// `lhs < rhs`
    LessThan(Box<Expression>, Box<Expression>),
    /// `lhs <= rhs`
    LessEqual(Box<Expression>, Box<Expression>),
    /// `lhs > rhs`
    GreaterThan(Box<Expression>, Box<Expression>),
    /// `lhs >= rhs`
    GreaterEqual(Box<Expression>, Box<Expression>),

    /// `lhs and rhs`
    LogicalAnd(Box<Expression>, Box<Expression>),

    /// `lhs or rhs`
    LogicalOr(Box<Expression>, Box<Expression>),
}

#[derive(Clone, Debug)]
pub enum Unary {
    /// `-x`
    Minus(Box<Expression>),
    /// `~x`
    BitwiseNot(Box<Expression>),
    /// `#x`
    Length(Box<Expression>),
    /// `not x`
    LogicalNot(Box<Expression>),
}

#[derive(Clone, Debug)]
pub struct ParameterList {
    pub names: Vec<String>,
    pub variadic: bool,
}
impl ParameterList {
    pub fn new(names: Vec<String>, variadic: bool) -> Self {
        Self { names, variadic }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionBody {
    pub parameters: ParameterList,
    pub block: statement::Block,
}
impl FunctionBody {
    pub fn new(parameters: Option<ParameterList>, block: statement::Block) -> Self {
        if let Some(p) = parameters {
            Self {
                parameters: p,
                block,
            }
        } else {
            Self {
                parameters: ParameterList::new(Vec::new(), false),
                block,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionCall {
    pub prefix: Box<Expression>,
    pub args: Vec<Expression>,
}
impl FunctionCall {
    pub fn new(prefix: Expression, args: Vec<Expression>) -> Self {
        Self {
            prefix: Box::new(prefix),
            args,
        }
    }
}

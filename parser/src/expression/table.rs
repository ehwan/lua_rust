use super::Expression;
use crate::Span;
use crate::SpannedString;

/// table field constructor [key] = `value`

#[derive(Clone, Debug)]
pub struct TableFieldKeyValue {
    pub key: Expression,
    pub value: Expression,
    pub span: Span,
}
impl TableFieldKeyValue {
    pub fn new(key: Expression, value: Expression, span: Span) -> Self {
        Self { key, value, span }
    }
    /// get the span of the table field
    pub fn span(&self) -> Span {
        self.span
    }
}

/// table field constructor `name = value`
#[derive(Clone, Debug)]
pub struct TableFieldNameValue {
    pub name: SpannedString,
    pub value: Expression,
    pub span: Span,
}
impl TableFieldNameValue {
    pub fn new(name: SpannedString, value: Expression, span: Span) -> Self {
        Self { name, value, span }
    }
    /// get the span of the table field
    pub fn span(&self) -> Span {
        self.span
    }
}

/// table field constructor `value`
#[derive(Clone, Debug)]
pub struct TableFieldValue {
    pub value: Expression,
}
impl TableFieldValue {
    pub fn new(value: Expression) -> Self {
        Self { value }
    }
}

/// table field
#[derive(Clone, Debug)]
pub enum TableField {
    /// `[key] = value`
    KeyValue(TableFieldKeyValue),
    /// `name = value`
    NameValue(TableFieldNameValue),
    /// `value`
    Value(TableFieldValue),
}

/// table constructor, a list of fields
#[derive(Clone, Debug)]
pub struct ExprTable {
    pub fields: Vec<TableField>,
    pub span: Span,
}
impl ExprTable {
    pub fn new(fields: Vec<TableField>, span: Span) -> Self {
        Self { fields, span }
    }
    /// get the span of the whole table constructor
    pub fn span(&self) -> Span {
        self.span
    }
}

use super::Expression;
use crate::Span;
use crate::SpannedString;

/// for internal use
#[derive(Clone, Debug)]
pub(crate) enum TableConstructorFieldBuilder {
    KeyValue(Expression, Expression),
    NameValue(SpannedString, Expression),
    Value(Expression),
}

/// table field for key-value pair.
/// syntax sugar like `name = value`, `value` will be converted to `["name"] = value`, `[counter] = value`.
#[derive(Clone, Debug)]
pub struct TableField {
    pub key: Expression,
    pub value: Expression,
    pub span: Span,
}
impl TableField {
    pub fn new(key: Expression, value: Expression, span: Span) -> Self {
        Self { key, value, span }
    }
    /// get the span of the table field
    pub fn span(&self) -> Span {
        self.span
    }
}

/// table constructor
#[derive(Clone, Debug)]
pub struct ExprTable {
    pub fields: Vec<TableField>,
    pub span: Span,
}
impl ExprTable {
    pub fn new(span: Span) -> Self {
        Self {
            fields: Vec::new(),
            span,
        }
    }
    /// get the span of the whole table constructor
    pub fn span(&self) -> Span {
        self.span
    }
}

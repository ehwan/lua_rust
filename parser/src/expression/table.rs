use super::Expression;

/// for internal use
#[derive(Clone, Debug)]
pub(crate) enum TableConstructorFieldBuilder {
    KeyValue(Expression, Expression),
    NameValue(String, Expression),
    Value(Expression),
}

/// table field for key-value pair.
/// syntax sugar like `name = value`, `value` will be converted to `["name"] = value`, `[counter] = value`.
#[derive(Clone, Debug)]
pub struct TableField {
    pub key: Expression,
    pub value: Expression,
}
impl TableField {
    pub fn new(key: Expression, value: Expression) -> Self {
        Self { key, value }
    }
}

/// table constructor
#[derive(Clone, Debug)]
pub struct ExprTable {
    pub fields: Vec<TableField>,
}
impl ExprTable {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }
}
